const fs=require('node:fs'),path=require('node:path');
function save(file,value){
  fs.mkdirSync(path.dirname(file),{recursive:true,mode:0o700});
  const fd=fs.openSync(file+'.tmp','w',0o600);
  try{fs.fchmodSync(fd,0o600);fs.writeFileSync(fd,typeof value==='string'?value:JSON.stringify(value,null,2)+'\n');fs.fsyncSync(fd);}finally{fs.closeSync(fd);}
  fs.renameSync(file+'.tmp',file);
}
function syncLedger(file,accounts,states,newPassword){
  let text=fs.readFileSync(file,'utf8');
  const begin='<!-- ACCOUNT_SUMMARY_START -->',end='<!-- ACCOUNT_SUMMARY_END -->';
  if(!text.includes(begin)||!text.includes(end))throw Error('Ledger summary markers missing');
  let rows=text.slice(text.indexOf(begin),text.indexOf(end)).split('\n').filter(l=>/^\| \d+ \|/.test(l));
  let next=Math.max(0,...rows.map(l=>Number(l.split('|')[1])))+1;
  for(const a of accounts){
    const s=states[a.email]||{};
    const verified=s.import?.verification?.success===true;
    const factorsVerified=s.awsSecurity?.status==='factors_and_relogin_verified';
    const importedPending=!verified&&s.importReceipt?.resourceId;
    const verifiedStatus=s.mailStatus==='verified'?'已改邮箱密码、已导入、模型验证通过':`已导入、模型验证通过；Outlook 未改密${factorsVerified?'；AWS 恢复邮箱＋MFA 已验证':''}`;
    const status=verified?verifiedStatus:importedPending?`已导入 prism01 #${importedPending}；模型验证待完成；${s.error||'待验证'}`:s.status==='skipped'?a.skipReason||'待确认，暂跳过':s.status==='failed'?`${s.stage} 受阻：${s.error}`:s.status==='interrupted'?`流程中断，待续跑：${s.error||s.stage}`:factorsVerified?'AWS 恢复邮箱＋MFA 已验证；Outlook 未改密；待导入':s.mailStatus==='verified'?'已改邮箱密码并验证；'+(s.stage||'待导入'):'待处理；'+(s.stage||'尚未开始');
    const existing=rows.find(l=>l.split('|')[2].trim()===a.email);
    const cells=existing?existing.split('|'):['',String(next++),a.email,'5',`卖家编号 ${a.vendorNumber}`,'—','',''];
    cells[5]=verified?s.import.id:importedPending?`${importedPending}（待验证）`:cells[5].trim();cells[6]=status;
    const row='| '+cells.slice(1,7).map(x=>String(x).trim()).join(' | ')+' |';
    if(existing){text=text.replace(existing,row);rows=rows.map(r=>r===existing?row:r);}else{text=text.replace(end,row+'\n'+end);rows.push(row);}
    const heading='### '+a.email+'\n';let start=text.indexOf(heading);
    if(start<0){text+='\n'+heading+'\n| 项目 | 内容 |\n| --- | --- |\n';start=text.indexOf(heading);}
    let stop=text.indexOf('\n### ',start+4);if(stop<0)stop=text.length;
    let section=text.slice(start,stop);
    const fields={
      'AWS / Kiro 账号':'`'+a.email+'`','AWS 原始密码':'`'+a.awsPassword+'`',
      'AWS 当前记录密码':'`'+a.awsPassword+'`'+(s.awsAuthenticated||verified?'（未改密；登录已验证）':'（未改密；尚未验证）'),
      'Outlook 账号':'`'+a.email+'`','Outlook 原始密码':'`'+a.mailPassword+'`',
      'Outlook 目标统一新密码':'`'+newPassword+'`',
      'Outlook 当前密码状态':s.mailStatus==='verified'?'`'+newPassword+'`（新会话登录验证通过）':s.mailStatus==='submitted'?'已提交改密，结果待验证；不能视为新密码已生效':'原始密码；尚未完成改密验证',
      '邮箱改密结果':s.mailStatus||'未提交','邮箱最近验证 / 提交时间':s.mailAt||'—',
      '导入结果':verified?`prism01 #${s.import.id}；${s.import.status} / ${s.import.health}`:status,
      '模型验证':verified?`${s.import.verification.model} 实际请求成功；${s.import.verification.totalMs} ms；${s.importAt}`:'尚未验证',
      '失败授权备份':s.credentialBackup?.directory||'—',
      '来源编号':`输入批次；卖家第 ${a.vendorNumber} 个（独立于台账序号）`,
    };
    for(const [label,value] of Object.entries(fields)){
      const prefix='| '+label+' |';const lines=section.split('\n');const i=lines.findIndex(l=>l.startsWith(prefix));
      const line=prefix+' '+value+' |';if(i>=0)lines[i]=line;else lines.push(line);section=lines.join('\n');
    }
    text=text.slice(0,start)+section.trimEnd()+'\n'+text.slice(stop);
  }
  const total=rows.length, imported=rows.filter(l=>/^\d+$/.test(l.split('|')[5].trim())).length;
  const oldRejected=rows.filter(l=>l.includes('邮箱密码被拒')).length;
  const failed=rows.filter(l=>l.includes('受阻：')).length;
  const skipped=rows.filter(l=>l.includes('暂跳过')).length;
  const unverified=rows.filter(l=>l.includes('模型验证待完成')).length;
  const pending=total-imported-oldRejected-failed-skipped-unverified;
  if(new Set(rows.map(l=>l.split('|')[2].trim())).size!==total)throw Error('Duplicate ledger emails');
  text=text.replace(/^按邮箱去重，共 .*$/m,`按邮箱去重，共 ${total} 个已提供账号：${imported} 个已导入 prism01 且模型验证通过、${unverified} 个已导入待验证、${oldRejected} 个此前邮箱密码被拒、${failed} 个本批处理受阻、${skipped} 个暂跳过、${pending} 个待处理。按实际邮箱统计，卖家编号不作为数量依据；原失败及补号记录保留。`);
  const stamp=new Date().toLocaleString('sv-SE',{timeZone:'Asia/Shanghai'});
  text=text.replace(/^更新于：[^。]+。.*$/m,`更新于：${stamp} CST。累计 ${total} 个去重账号；后续新增继续追加到本文件。`)
    .replace(/全部 \d+ 个账号/g,`全部 ${total} 个账号`);
  save(file,text);
  return {total,imported,unverified,oldRejected,failed,skipped,pending};
}

module.exports={syncLedger};
