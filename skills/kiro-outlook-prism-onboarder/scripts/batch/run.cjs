const fs=require('fs'),path=require('path'),{promisify}=require('util');
const exec=promisify(require('child_process').execFile);
// Sanitized source copy. Requires the existing helper scripts; not a standalone program.
const required=name=>{if(!process.env[name])throw Error('Missing environment variable: '+name);return process.env[name];};
const base=path.join(__dirname,'scripts')+path.sep;
process.env.KIRO_RUN_DIR=path.join(required('BATCH_RUN_DIR'),'import-run');
const {chromium}=require('playwright');
const {auth,totp,storeSecret,readSecrets,enrollMfa,changePassword,freshMailCode}=require(base+'kiro_builder_security.cjs');
const {login,changeAndVerify,submitPasswordChange}=require(base+'outlook_change_password.cjs');
const {outlookCodes,oldEmailCode,mailbox}=require(base+'kiro_change_builder_email.cjs');
const {startDevice,awsLogin}=require(base+'kiro_builder_assisted.cjs');
const {createRotator}=require(base+'kiro_clash_rotate.cjs');
const root=path.resolve(required('BATCH_RUN_DIR')),read=f=>fs.existsSync(f)?JSON.parse(fs.readFileSync(f)):{};
const accounts=read(root+'/accounts.json'),original=read(root+'/mfa-secrets.json');
const states=read(root+'/states.json'),seedFile=root+'/replacement-mfa.json';
const source=fs.readFileSync(base+'kiro_builder_security.cjs','utf8');
const PROFILE=source.match(/const PROFILE = '([^']+)'/)[1],SECURITY=source.match(/const SECURITY = '([^']+)'/)[1];
const proxy=process.env.KIRO_PROXY||'http://127.0.0.1:7890';
const target=fs.readFileSync(required('MAILBOX_PASSWORD_FILE'),'utf8').trim();
process.umask(0o077);
const save=()=>{fs.writeFileSync(root+'/states.json.tmp',JSON.stringify(states,null,2),{mode:0o600});fs.renameSync(root+'/states.json.tmp',root+'/states.json');require('child_process').execFileSync(process.execPath,[path.join(__dirname,'sync-ledger.cjs')],{timeout:30000});};
const log=(email,event)=>console.log(JSON.stringify({at:new Date().toISOString(),email,event}));
const backend=async(...args)=>{const result=await exec('python3',[base+'kiro_builder_backend.py',...args],{env:{...process.env,KIRO_RUN_DIR:root+'/import-run'},timeout:360000,maxBuffer:1048576});return result.stdout;};
async function main(){
 const preflight=read(root+'/preflight.json');
 if(preflight.node!==required('KIRO_IMPORT_BASE')||accounts.some(a=>!Array.isArray(preflight.accounts?.[a.email])||preflight.accounts[a.email].length))throw Error('Preflight incomplete or duplicate accounts');
 const rotation=createRotator();await rotation.check();
 const browser=await chromium.launch({channel:'chrome',headless:false,proxy:{server:proxy},args:['--remote-debugging-port=9448']});
 for(const account of accounts){
  if(process.env.RETRY_EMAILS&&!process.env.RETRY_EMAILS.split(',').includes(account.email))continue;
  if(fs.existsSync(root+'/stop-after-current'))break;
  const email=account.email,s=states[email]||={};if(s.verified||s.failed){log(email,'SKIPPED');continue;}
  await rotation.beforeAccount();
  let context,device,verificationContext;
  const accountContexts=new Set();
  const checkpoint=stage=>{s.stage=stage;save();log(email,stage);};
  const seed=()=>readSecrets(seedFile)[email]?.secret||original[email].secret;
  const otp=async()=>{const left=30-Math.floor(Date.now()/1000)%30;if(left<15)await new Promise(r=>setTimeout(r,(left+.1)*1000));return totp(seed());};
  let password=s.passwordChanged?target:account.awsPassword,mail,baseline,page;
  const authenticate=async(p,done,pw=password)=>auth(p,{email,password:pw,getTotp:otp,getMailCode:()=>oldEmailCode(mail,baseline),done,notify:e=>log(email,e)});
  const fresh=async()=>{
   const verification=await browser.newContext({locale:'en-US'});accountContexts.add(verification);
   const p=await verification.newPage(),stages=[];
   await p.goto(PROFILE,{waitUntil:'domcontentloaded'});
   await auth(p,{email,password,getTotp:otp,getMailCode:()=>oldEmailCode(mail,baseline),done:['profile'],notify:e=>{stages.push(e);log(email,e);}});
   if(!stages.includes('AUTH_STAGE: password')||!stages.includes('AUTH_STAGE: mfa_login'))throw Error('Fresh password/MFA login not exercised');
   if(!(await p.locator('body').innerText()).toLowerCase().includes(email))throw Error('Fresh identity mismatch');
   const previous=verificationContext;verificationContext=verification;
   page=p;s.freshVerifiedAt=new Date().toISOString();s.freshVerifiedCredential={passwordVersion:s.passwordChanged?'target':'original',mfaVersion:readSecrets(seedFile)[email]?'replacement':'original'};if(readSecrets(seedFile)[email])storeSecret(seedFile,email,{secret:seed(),status:'login_verified',verifiedAt:s.freshVerifiedAt});save();
   if(previous) {await previous.close();accountContexts.delete(previous);}
  };
  try{
   context=await browser.newContext({locale:'en-US'});accountContexts.add(context);context.setDefaultTimeout(20000);
   checkpoint('outlook_password');
   const receipt=path.join(root,email+'-outlook.json');
   const recovery=read(root+'/recovery-state.json');
   if(recovery.email===email&&recovery.status==='recovery_verified'){
    mail=await context.newPage();await login(mail,email,account.mailPassword,'https://outlook.office.com/mail/');s.outlookUnchanged=true;save();
   }else try{await changeAndVerify({email,receipt,alreadyStandard:account.mailPassword===target,
    change:async beforeSubmit=>{const p=await context.newPage();try{await login(p,email,account.mailPassword,'https://myaccount.microsoft.com/');await submitPasswordChange(p,{email,oldPassword:account.mailPassword,newPassword:target,beforeSubmit,verificationTimeoutMs:1});}finally{await p.close();}},
    verify:async()=>{mail=await context.newPage();await login(mail,email,target,'https://outlook.office.com/mail/');}});}catch(error){
     if(!/Authenticator approval timed out|additional approval|verification timed out/i.test(error.message))throw error;
     s.outlookUnchanged=true;s.outlookBlocked=String(error.message).split('\n')[0];save();
     mail=await context.newPage();await login(mail,email,account.mailPassword,'https://outlook.office.com/mail/');
   }
   if(read(receipt).status==='verified'){s.outlookUnchanged=false;delete s.outlookBlocked;save();}
   s.outlookVerified=true;checkpoint('aws_security');baseline=await outlookCodes(mail);page=await context.newPage();
   await page.goto(PROFILE,{waitUntil:'domcontentloaded'});await authenticate(page,['profile']);
   if(!(await page.locator('body').innerText()).toLowerCase().includes(email))throw Error('Profile identity mismatch');
   await page.goto(SECURITY,{waitUntil:'domcontentloaded'});await authenticate(page,['security']);
   const rows=page.getByRole('row').filter({hasText:'TOTP'});await rows.first().waitFor();const names=await rows.allTextContents();
   if(s.outlookUnchanged&&!s.recoveryVerified){
    checkpoint('aws_recovery_email');
    const crypto=require('crypto');
    const apiBase=required('MAIL_API_BASE'),token=fs.readFileSync(required('MAIL_ADMIN_TOKEN_FILE'),'utf8').trim();
    const api=(route,data)=>mailbox(apiBase,token,route,data);
    if(recovery.email===email&&recovery.status==='recovery_verified'){s.recoveryEmail=recovery.alias.alias_email;s.recoveryVerified=true;save();}
    else{
     if(!s.recoveryAlias){const id='kiro-migration-'+crypto.createHash('sha256').update(email).digest('hex').slice(0,24);const result=await api('/aliases/batch',{account_ids:[id]});s.recoveryAlias=result.aliases?.find(x=>x.account_id===id&&x.status==='active');if(!s.recoveryAlias)throw Error('Recovery alias missing');save();}
     if(!s.recoverySubmitted){await page.getByRole('button',{name:/^(edit|add)-recovery-email-button$/}).click();await authenticate(page,['recovery_form']);await page.getByRole('textbox',{name:'Recovery email',exact:true}).fill(s.recoveryAlias.alias_email);s.recoverySince=new Date().toISOString();s.recoverySubmitted=true;save();await page.getByRole('button',{name:/^Continue$|^继续$/}).click();}
     await auth(page,{email,password,getTotp:otp,done:['security'],transitionFrom:'recovery_form',getMailCode:async()=>{for(let n=0;n<80;n++){const data=await api('/mailboxes/'+encodeURIComponent(s.recoveryAlias.alias_email)+'/messages?limit=20');const code=freshMailCode(data.messages||[],s.recoverySince,s.recoveryAlias.alias_email);if(code)return code;await new Promise(r=>setTimeout(r,3000));}throw Error('Recovery OTP timeout');}});
     await page.getByText(s.recoveryAlias.alias_email,{exact:true}).waitFor();s.recoveryEmail=s.recoveryAlias.alias_email;s.recoveryVerified=true;save();
    }
   }
   s.originalMfaCount??=names.length;save();
   if(s.originalMfaCount===1&&!s.mfaRotated){
    if(!readSecrets(seedFile)[email]){
    if(names.length!==1)throw Error('MFA count changed; inspect');
    s.oldMfaName=(await rows.first().innerText()).split('\n').map(t=>t.trim()).find(t=>t.includes('MFA'));save();
    checkpoint('mfa_register_open');await page.getByTestId('mfa-register-button').click();
    await authenticate(page,['mfa_choice']);
    await page.getByText(/Authenticator app|身份验证器应用程序/,{exact:true}).click();
    await page.getByRole('button',{name:/^Next$|^下一步$/}).click();
    await page.getByText(/Show secret key|显示密钥/,{exact:true}).click();
    const keys=[...new Set((await page.locator('body').innerText()).match(/\b[A-Z2-7]{32,128}\b/g)||[])];
    if(keys.length!==1)throw Error('Seed ambiguous');
    storeSecret(seedFile,email,{secret:keys[0],status:'pending_registration',oldMfaName:s.oldMfaName});
    await enrollMfa(page,{secret:seed(),getTotp:otp,beforeSubmit:()=>checkpoint('mfa_enrollment_submitted')});
    storeSecret(seedFile,email,{secret:seed(),status:'registered_pending_login'});
    }
    await page.goto(SECURITY,{waitUntil:'domcontentloaded'});await authenticate(page,['security']);
   }else if(!s.mfaRotated&&names.length!==2)throw Error('Unexpected MFA count');
   checkpoint('aws_password');
   if(!s.passwordChanged){
    if(s.passwordSubmitted)throw Error('Password submission uncertain; reconcile first');
    await page.getByTestId('change-password-button').click();await authenticate(page,['new_password']);
    await changePassword(page,target,()=>{s.passwordSubmitted=true;save();});s.passwordChanged=true;password=target;save();
   }
   if(!s.sessionsVerified){
    checkpoint('aws_sessions');
    await page.goto(SECURITY,{waitUntil:'domcontentloaded'});await authenticate(page,['security']);
    if(!s.sessionsRevokedAt){
     if(s.sessionsDeleteSubmittedAt){
      await page.getByText(/Signed in devices|登录设备/).first().waitFor();
      const observed=(await page.locator('body').innerText()).match(/(?:Signed in devices|登录设备)\s*\((\d+)\)/)?.[1];
      if(observed===undefined)throw Error('Cannot reconcile previous session deletion');
      s.sessionReconciliation={at:new Date().toISOString(),count:Number(observed)};save();
      if(Number(observed)<=1){s.sessionsRevokedAt=s.sessionReconciliation.at;save();}
      else{delete s.sessionsDeleteSubmittedAt;save();}
     }
     if(!s.sessionsRevokedAt){
     await page.getByRole('button',{name:'show-Delete all active sessions-modal',exact:true}).click();
     const confirmation=page.getByRole('textbox',{name:/delete all/i});
     const confirmButton=page.getByRole('button',{name:'confirm-delete-all-sessions',exact:true});
     let ready=false;
     for(let attempt=0;attempt<3;attempt++){
      await confirmation.fill('delete all');await new Promise(r=>setTimeout(r,1000));
      if(await confirmation.inputValue()==='delete all'&&await confirmButton.isEnabled()){ready=true;break;}
     }
     if(!ready)throw Error('Session confirmation did not settle; no deletion submitted');
     s.sessionsDeleteSubmittedAt=new Date().toISOString();save();
     try{await confirmButton.click({timeout:5000});}catch(error){
      if(await confirmButton.isVisible().catch(()=>false)&&!await confirmButton.isEnabled())delete s.sessionsDeleteSubmittedAt;
      save();throw error;
     }
     await page.waitForURL(u=>new URL(u).hostname==='us-east-1.sso.signin.aws',{timeout:60000});
     s.sessionsRevokedAt=new Date().toISOString();save();
     }
    }
    await fresh();
    await page.goto(SECURITY,{waitUntil:'domcontentloaded'});await authenticate(page,['security']);
    await page.getByText(/Signed in devices|登录设备/).first().waitFor();
    const text=await page.locator('body').innerText(),count=text.match(/(?:Signed in devices|登录设备)\s*\((\d+)\)/)?.[1];
    if(count===undefined)throw Error('Session count unavailable');
    s.activeSessionCount=Number(count);save();
    if(s.activeSessionCount>1)throw Error('Session revocation propagation pending; no repeated deletion');
    s.sessionsVerified=true;save();
   }
   if(s.originalMfaCount===1&&!s.mfaRotated){
    if(!s.sessionsVerified||s.freshVerifiedCredential?.passwordVersion!=='target'||s.freshVerifiedCredential?.mfaVersion!=='replacement')throw Error('New password and replacement MFA must pass post-revocation login before old MFA deletion');
    await page.goto(SECURITY,{waitUntil:'domcontentloaded'});await authenticate(page,['security']);
    const oldRow=page.getByRole('row').filter({hasText:s.oldMfaName});await oldRow.first().waitFor();if(await oldRow.count()!==1)throw Error('Old MFA identity ambiguous');
    await oldRow.getByRole('radio').check();await page.getByTestId('mfa-delete-button').click();
    checkpoint('old_mfa_delete_submitted');await page.getByRole('button',{name:'confirm-delete',exact:true}).click();
    await oldRow.waitFor({state:'detached'});s.mfaRotated=true;storeSecret(seedFile,email,{secret:seed(),status:'rotated',verifiedAt:s.freshVerifiedAt});save();
   }
   checkpoint('kiro_device');
   device=await startDevice();await awsLogin(page,mail,{...account,awsPassword:password,mfaSecret:seed()},baseline,device);device=null;
   checkpoint('prism_import');await backend('import',email);
   const receiptImport=read(root+'/import-run/verified-imports.json').accounts?.find(r=>r.email===email);
   if(!receiptImport?.verification?.success)throw Error('Import verification missing');
   s.verified=true;s.import=receiptImport;save();await backend('detach',email);log(email,'VERIFIED #'+receiptImport.id);
  }catch(error){
   if(device&&!device.state().done)try{process.kill(-device.child.pid,'SIGTERM');}catch{}
   let detail=String(error.stack||error);for(const secret of [account.mailPassword,account.awsPassword,target,seed()])detail=detail.split(secret).join('[redacted]');fs.writeFileSync(root+'/'+email+'-error.txt',detail,{mode:0o600});
   s.failed=true;s.error=String(error.message).split('\n')[0];for(const secret of [account.mailPassword,account.awsPassword,target,seed()])s.error=s.error.split(secret).join('[redacted]');save();log(email,'FAILED '+s.error);
   const who=await exec('kiro-cli',['whoami']).catch(()=>({stdout:''}));if(who.stdout.includes('Email: '+email)){await backend('preserve-failed',email);}
  }finally{for(const owned of accountContexts)await owned.close();}
  if(s.verified&&!s.failed)rotation.completed();
 }
 await browser.close();log(null,'BATCH_COMPLETE');
}
main().catch(e=>{console.error(String(e.message).split('\n')[0]);process.exitCode=1;});
