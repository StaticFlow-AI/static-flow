const fs=require('fs'),crypto=require('crypto');
const {syncLedger}=require('./scripts/ledger-core.cjs');
const root=process.env.BATCH_RUN_DIR,file=process.env.ACCOUNT_LEDGER_FILE;
if(!root||!file||!process.env.MAILBOX_PASSWORD_FILE)throw Error('Missing ledger configuration');
const read=f=>fs.existsSync(f)?JSON.parse(fs.readFileSync(f)):{};
const target=fs.readFileSync(process.env.MAILBOX_PASSWORD_FILE,'utf8').trim();
process.umask(0o077);
function sync(){
 const accounts=read(root+'/accounts.json'),states=read(root+'/states.json'),original=read(root+'/mfa-secrets.json'),replacement=read(root+'/replacement-mfa.json');
 for(const a of accounts){const s=states[a.email]||={};const receipt=read(root+'/'+a.email+'-outlook.json');if(receipt.status){s.mailStatus=receipt.status;s.mailAt=receipt.at;}const imported=read(root+'/import-run/verified-imports.json').accounts?.find(r=>r.email===a.email&&r.verification?.success);const pendingImport=read(root+'/import-run/'+a.email+'-import.json').items?.find(x=>x.status==='succeeded'&&x.resourceId);if(pendingImport&&!imported)s.importReceipt=pendingImport;const remoteFailure=read(root+'/remote-verification-failures.json')[a.email];if(remoteFailure&&!imported)s.error=remoteFailure.error;const archived=read(root+'/import-run/'+a.email+'-pending.json');if(archived.directory)s.credentialBackup=archived;if(imported){s.import=imported;s.verified=true;s.importAt=imported.verificationAt||new Date().toISOString();}s.status=s.verified?'verified':s.failed?'failed':s.stage?'running':'pending';}
 const backup=root+'/ledger-before-20261028.md';if(!fs.existsSync(backup))fs.copyFileSync(file,backup);
 syncLedger(file,accounts,states,target);
 let text=fs.readFileSync(file,'utf8');
 for(const a of accounts){
  const s=states[a.email],record=replacement[a.email],recovery=read(root+'/recovery-state.json');const start='<!-- OCT28_SECURITY:'+a.email+':START -->',end='<!-- OCT28_SECURITY:'+a.email+':END -->';
  const block=[start,'### 2026-10-28 密码与 MFA 记录：'+a.email,'',
   '- AWS 原始密码：`'+a.awsPassword+'`。',
   '- AWS 目标新密码：`'+target+'`。状态：'+(s.passwordChanged?'修改成功提示已确认':s.passwordSubmitted?'已提交，待核对，禁止重复提交':'尚未提交')+'。',
   '- AWS 当前已确认密码：`'+(s.passwordChanged?target:a.awsPassword)+'`；新会话验证时间：'+(s.freshVerifiedAt||'尚未验证')+'。',
   '- Outlook 原始密码：`'+a.mailPassword+'`；目标新密码：`'+target+'`；回执状态：'+(s.mailStatus||'未提交')+'。',
   '- 原 AWS TOTP 密钥：`'+original[a.email].secret+'`（供应商提供，永久保留历史）。',
   '- 原 MFA 数量：'+(s.originalMfaCount??'尚未核对')+'；旧设备：'+(s.oldMfaName||'见安全页核对')+'。',
   '- 新 AWS TOTP 密钥：'+(record?'`'+record.secret+'`；登记状态：'+record.status:'尚未生成')+'。',
   '- MFA 更换状态：'+(s.mfaRotated?'旧设备已移除，新设备已验证':record?'新设备流程进行中，旧设备处理见状态记录':s.originalMfaCount===2?'原有两个设备，按用户要求保持原样':'待处理')+'。',
   '- MFA 主文件：`'+root+'/replacement-mfa.json`；独立备份：`'+root+'/replacement-mfa.json.backup.json`。',
   '- AWS 恢复邮箱：'+(s.recoveryEmail?'`'+s.recoveryEmail+'`；已验证':recovery.email===a.email?'`'+recovery.alias.alias_email+'`；状态：'+recovery.status:'未由本批修改')+'。',
   '- AWS 全部会话清理：'+(s.sessionsVerified?'删除后复登已验证；设备数 '+s.activeSessionCount:s.sessionsRevokedAt?'删除已观察到退出；复登／数量待验证':s.sessionsDeleteSubmittedAt?'已提交，结果待核对':'尚未执行（此前导入成功不代表已踢出旧会话）')+'。',
   '- Prism：'+(s.import?'#'+s.import.id+'；模型验证：'+s.import.verification?.success:s.importReceipt?'#'+s.importReceipt.resourceId+'；已导入，模型验证未通过':'尚未确认导入')+'；阶段：'+(s.stage||'未开始')+'；错误：'+(s.error||'无')+'。',end].join('\n');
  const begin=text.indexOf(start),finish=text.indexOf(end);text=begin<0?text+'\n\n'+block+'\n':text.slice(0,begin)+block+text.slice(finish+end.length);
  const heading='### '+a.email+'\n',beginAccount=text.indexOf(heading);if(beginAccount>=0){let stop=text.indexOf('\n### ',beginAccount+heading.length);if(stop<0)stop=text.length;const section=text.slice(beginAccount,stop).replace(/^\| AWS 当前记录密码 \|.*$/m,'| AWS 当前记录密码 | `'+(s.passwordChanged?target:a.awsPassword)+'`（'+(s.passwordChanged?'已改密；独立登录验证时间 '+(s.freshVerifiedAt||'待验证'):'原始密码，未改密')+'） |').replace(/^\| 来源编号 \|.*$/m,'| 来源编号 | 输入批次；批内第 '+a.vendorNumber+' 个 |');text=text.slice(0,beginAccount)+section+text.slice(stop);}
 }
 fs.writeFileSync(file+'.tmp',text,{mode:0o600});fs.renameSync(file+'.tmp',file);
 const check=fs.readFileSync(file,'utf8');for(const a of accounts){if(!check.includes(original[a.email].secret)||replacement[a.email]&&!check.includes(replacement[a.email].secret))throw Error('Ledger secret read-back failed');}
 fs.writeFileSync(root+'/ledger-sync.json',JSON.stringify({at:new Date().toISOString(),accounts:accounts.length,sha256:crypto.createHash('sha256').update(check).digest('hex')}),{mode:0o600});
}
sync();if(process.argv.includes('--watch'))setInterval(()=>{try{sync();}catch(e){console.error('LEDGER_SYNC_FAILED: '+e.message);}},5000);
