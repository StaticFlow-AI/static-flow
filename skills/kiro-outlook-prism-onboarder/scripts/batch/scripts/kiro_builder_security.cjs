#!/usr/bin/env node
// Builder ID hardening with durable seed checkpoints and fresh-login verification.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { chromium } = require('playwright');
const { login: outlookLogin } = require('./outlook_change_password.cjs');
const { createEmailRetry } = require('./kiro_email_retry.cjs');
const { save, mailbox, outlookCodes, oldEmailCode } = require('./kiro_change_builder_email.cjs');
const PROFILE = 'https://profile.aws.amazon.com/';
const SECURITY = 'https://us-east-1.credentials.signin.aws/#/security';
const sleep = ms => new Promise(r => setTimeout(r, ms));
const now = () => new Date().toISOString();
const read = file => fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, 'utf8')) : {};
const body = page => page.locator('body').innerText();

// Seed writes are durable and independently recoverable before AWS enrollment.
function durableJson(file, data) {
  fs.mkdirSync(path.dirname(file), {recursive:true,mode:0o700});
  const tmp = file + '.tmp';
  const fd = fs.openSync(tmp, 'w', 0o600);
  try { fs.fchmodSync(fd,0o600); fs.writeFileSync(fd,JSON.stringify(data,null,2)+'\n'); fs.fsyncSync(fd); }
  finally { fs.closeSync(fd); }
  fs.renameSync(tmp,file);
  const dir = fs.openSync(path.dirname(file),'r');
  try { fs.fsyncSync(dir); } finally { fs.closeSync(dir); }
}
function readSecrets(file) {
  const backup = read(file+'.backup.json'), primary = read(file);
  for (const [email,record] of Object.entries(primary)) {
    if (backup[email] && backup[email].secret !== record.secret) throw new Error('MFA primary/backup seed mismatch; inspect without overwriting');
  }
  return {...backup,...primary};
}
function storeSecret(file,email,record) {
  totp(record.secret,59); // Validate before either copy is touched.
  const all = readSecrets(file);
  if (all[email] && all[email].secret !== record.secret) throw new Error('Refusing to overwrite an existing MFA seed');
  all[email] = {...all[email],...record};
  durableJson(file+'.backup.json',all);
  durableJson(file,all);
  for (const copy of [file,file+'.backup.json']) {
    if (read(copy)[email]?.secret !== record.secret || (fs.statSync(copy).mode & 0o777)!==0o600)
      throw new Error('MFA seed read-back/permission verification failed');
  }
}

function totp(secret, seconds = Math.floor(Date.now() / 1000), digits = 6) {
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
  const clean = secret.replace(/\s|=+$/g, '').toUpperCase();
  if (!/^[A-Z2-7]{16,128}$/.test(clean)) throw new Error('Invalid TOTP secret');
  let bits = 0, value = 0; const bytes = [];
  for (const char of clean) {
    value = (value << 5) | alphabet.indexOf(char); bits += 5;
    if (bits >= 8) { bits -= 8; bytes.push((value >>> bits) & 255); }
  }
  const counter = Buffer.alloc(8); counter.writeBigUInt64BE(BigInt(Math.floor(seconds / 30)));
  const mac = crypto.createHmac('sha1', Buffer.from(bytes)).update(counter).digest();
  return String((mac.readUInt32BE(mac[19] & 15) & 0x7fffffff) % (10 ** digits)).padStart(digits, '0');
}
function freshMailCode(messages, since, email) {
  const candidates = messages.filter(m => /^(?:.*<)?no-reply@login\.awsapps\.com>?$/i.test(m.sender || '') &&
    /^Verify your identity$/i.test(m.subject || '') && Date.parse(m.received_at) >= Date.parse(since) &&
    (!m.recipient || m.recipient.toLowerCase().includes(email.toLowerCase())))
    .sort((a, b) => Date.parse(b.received_at) - Date.parse(a.received_at));
  if (!candidates.length) return null;
  const m = candidates[0];
  const text = (m.body_text || '') + ' ' + (m.body_html || '')
    .replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, ' ').replace(/<[^>]+>/g, ' ').replace(/&nbsp;|&#160;/g, ' ');
  const codes = [...new Set(text.match(/\b\d{6}\b/g) || [])];
  return codes.length === 1 ? codes[0] : null;
}
function stage(text, url) {
  const u = new URL(url);
  if (u.protocol !== 'https:' || !['profile.aws.amazon.com', 'us-east-1.credentials.signin.aws', 'us-east-1.sso.signin.aws'].includes(u.hostname))
    throw new Error('Unexpected AWS destination');
  if (/Change your password|更改您的密码/.test(text)) return 'new_password';
  if (/successfully registered|已成功注册/.test(text)) return 'mfa_registered';
  if (/Set\s*up (?:your |an? )?authenticator app|设置身份验证器应用程序/i.test(text)) return 'mfa_setup';
  if (/Security check|Security Verification|Invalid captcha/i.test(text) && !/successfully solved|Success!/i.test(text)) return 'captcha';
  if (/ERR-\d+|incorrect.*password|invalid.*code|code.*expired/i.test(text)) return 'error';
  if (u.hostname === 'us-east-1.credentials.signin.aws' && /Signed in devices|登录设备/.test(text)) return 'security';
  if (u.hostname === 'profile.aws.amazon.com' && /Contact information/.test(text)) return 'profile';
  if (/Register (?:an? )?MFA device|注册 MFA 设备/i.test(text)) return 'mfa_choice';
  if (/Add recovery email/.test(text)) return 'recovery_form';
  if (/Verify your identity/.test(text)) return 'email_otp';
  if (/Additional verification required|需要额外验证|MFA code|MFA 代码/.test(text)) return 'mfa_login';
  if (/Sign in with your AWS Builder ID/.test(text)) return 'password';
  if (/Get started/.test(text)) return 'email';
  return 'loading';
}
async function auth(page, config) {
  const { email, password, getMailCode, getTotp, done, notify = console.log, timeout = 600000 } = config;
  let previous = '', sent = false, identityDeadline = 0;
  const emailRetry=createEmailRetry({delayMs:config.emailRetryDelayMs??8000,notify,mode:config.emailContinueMode});
  const submitted = new Set();
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline + emailRetry.pausedMs()) {
    const decline=page.getByRole('button',{name:'Decline',exact:true});
    if(await decline.isVisible().catch(()=>false))await decline.click();
    const text = await body(page), current = stage(text, page.url());
    if(/Enter your name|Create your password/.test(text))throw new Error('AWS opened signup; existing-account login stopped');
    if (current !== previous) { sent = submitted.has(current); previous = current; identityDeadline = 0; notify('AUTH_STAGE: ' + current); }
    if (done.includes(current)) return current;
    if(await emailRetry.handle(page,text,email))continue;
    if (current === config.transitionFrom) { await sleep(1000); continue; }
    // Credential setup belongs to the explicit checkpointed handlers below.
    if (['new_password', 'mfa_setup', 'mfa_registered', 'mfa_choice', 'recovery_form'].includes(current))
      throw new Error('Human/setup stage reached outside its explicit handler');
    if (current === 'error') {
      throw new Error('AWS authentication rejected ' + (text.match(/ERR-\d+/)?.[0] || 'credentials/code'));
    }
    if (current === 'captcha') {
      if (!sent) { notify('HUMAN_REQUIRED: complete CAPTCHA; automatic actions paused'); await page.bringToFront(); sent = true; }
    } else if (!sent && current === 'email') {
      await page.getByRole('textbox', {name:/^Email$/i}).fill(email);
      await page.getByRole('button', {name:/^Continue$|^继续$/}).click(); emailRetry.submitted(); sent = true; submitted.add(current);
    } else if (!sent && current === 'password') {
      // AWS can render the password heading before the account identity arrives.
      // Wait briefly, but never fill a password without the expected identity.
      if (!text.toLowerCase().includes(email.toLowerCase())) {
        identityDeadline ||= Date.now() + (config.identityTimeoutMs ?? 15000);
        if (Date.now() >= identityDeadline) throw new Error('AWS account identity mismatch');
        await sleep(1000); continue;
      }
      await page.locator('input[type=password]').fill(password);
      await page.getByRole('button', {name:/^Continue$|^继续$/}).click(); sent = true; submitted.add(current);
    } else if (!sent && current === 'email_otp') {
      if (!getMailCode) throw new Error('Email verification requires available mailbox');
      await page.getByRole('textbox', {name:/Verification code|验证码/i}).fill(await getMailCode());
      await page.getByRole('button', {name:/^Continue$|^继续$/}).click(); sent = true; submitted.add(current);
    } else if (!sent && current === 'mfa_login') {
      if (!getTotp) throw new Error('Existing MFA secret is unavailable');
      await page.getByRole('textbox', {name:/MFA code|MFA 代码/i}).fill(await getTotp());
      await page.getByRole('button', {name:/^Continue$|^继续$/}).click(); sent = true; submitted.add(current);
    }
    await sleep(1000);
  }
  throw new Error('Authentication/human step timed out');
}
function securitySnapshot(text) {
  const recovery = text.match(/(?:Recovery email|恢复电子邮件地址)\s+([^\s]+@[^\s]+)/)?.[1] || null;
  const sessions = text.match(/(?:Signed in devices|登录设备)\s*\((\d+)\)/)?.[1];
  if (sessions === undefined) throw new Error('Security page not fully loaded');
  return { recoveryEmail: recovery, hasTotp: /\bTOTP\b/.test(text), sessions: Number(sessions) };
}
function upsert(text, start, end, block) {
  const a = text.indexOf(start), b = text.indexOf(end);
  if ((a < 0) !== (b < 0) || (a >= 0 && b < a)) throw new Error('Invalid ledger markers');
  return a < 0 ? text.trimEnd() + '\n\n' + block + '\n' : text.slice(0, a) + block + text.slice(b + end.length);
}
function ledger(file, email, s, password, secret) {
  let text = fs.readFileSync(file, 'utf8');
  if (!text.includes(email)) throw new Error('Account missing from ledger');
  const start = `<!-- AWS_SECURITY:${email}:START -->`, end = `<!-- AWS_SECURITY:${email}:END -->`;
  const complete = ['hardened_and_relogin_verified','factors_and_relogin_verified'].includes(s.status);
  const factorsOnly = s.status === 'factors_and_relogin_verified' || s.passwordUnchanged === true;
  const imported = s.prismImport?.verification?.success === true;
  const block = `${start}\n### AWS 安全加固：${email}\n\n` +
    `- 阶段：${s.status}；更新时间（UTC）：${s.updatedAt}。\n` +
    `- 恢复邮箱：\`${s.recoveryEmail || '未设置'}\`；状态：${s.recoveryEmailStatus || '未验证'}。\n` +
    `- MFA：${s.mfaStatus || '未验证'}。\n` +
    `- AWS 密码：${s.passwordStatus || '尚未处理'}。${s.freshLoginVerifiedAt ? (factorsOnly?'已验证现有密码＋MFA 登录；密码未更改。':'已退出后验证通用密码＋MFA 登录。') : '尚未完成复登验证。'}\n` +
    `- 会话清理：${s.sessionsRevokeStatus || '未执行'}；复登验证时设备数：${s.activeSessionCountAfterVerification ?? '未核对'}。\n` +
    `- 主邮箱未由此工具修改；Outlook 密码未修改；${imported?`prism01 #${s.prismImport.id} 已导入且模型验证通过`:'导入状态见账号汇总，本安全步骤不执行导入'}。${factorsOnly?'本次未更改密码或删除会话。':'AWS 会话撤销完全生效最多可能需一小时。'}\n${end}`;
  text = upsert(text, start, end, block);
  if (secret) {
    const a = `<!-- AWS_MFA_SECRET:${email}:START -->`, b = `<!-- AWS_MFA_SECRET:${email}:END -->`;
    text = upsert(text, a, b, `${a}\n### MFA 密钥：${email}（私密）\n\n- TOTP 密钥：\`${secret}\`。\n- 参数：SHA1 / 6 位 / 30 秒。状态：${s.mfaStatus || '待绑定'}。\n${b}`);
  }
  if (password && s.passwordStatus && !factorsOnly) {
    const a = `<!-- AWS_PASSWORD_PENDING:${email}:START -->`, b = `<!-- AWS_PASSWORD_PENDING:${email}:END -->`;
    text = upsert(text, a, b, `${a}\n### AWS 通用密码：${email}（${s.freshLoginVerifiedAt ? '已验证' : '待设置／核对'}）\n\n- 通用密码：\`${password}\`。\n- 状态：${s.passwordStatus}；未验证前不能视为当前密码。原始密码保留作历史记录。\n${b}`);
  }
  if (complete) {
    const lines = text.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const cells = lines[i].split('|');
      if (cells.length === 8 && cells[2].trim() === email) {
        if(imported)cells[5]=' '+s.prismImport.id+' ';
        const importedText=imported?'已导入，模型验证通过':'待导入';
        lines[i] = cells.slice(0, 6).join('|') + (factorsOnly?`| AWS 恢复邮箱＋TOTP MFA 已启用；现有密码＋MFA 复登通过；${importedText} |`:`| AWS 恢复邮箱＋TOTP MFA 已启用；通用密码复登通过；旧会话已清理，剩当前 1 个；${importedText} |`);
      }
    }
    text = lines.join('\n');
    const a = text.indexOf('### ' + email + '\n');
    if (a >= 0) {
      let b = text.indexOf('\n### ', a + 4); if (b < 0) b = text.indexOf('\n<!--', a + 4); if (b < 0) b = text.length;
      const part = text.slice(a, b).replace(/^\| AWS 当前记录密码 \|.*$/m,
        () => `| AWS 当前记录密码 | \`${password}\`（${factorsOnly?'现有密码，未改密':'通用密码'}＋TOTP MFA 重新登录验证通过） |`)
        .replace(/^\| 导入结果 \|.*$/m,imported?`| 导入结果 | prism01 #${s.prismImport.id}；${s.prismImport.status} / ${s.prismImport.health}；模型验证通过 |`:`| 导入结果 | 待导入；AWS 恢复邮箱＋MFA 已启用，${factorsOnly?'现有':'通用'}密码＋MFA 复登已通过 |`)
        .replace(/^\| 已分配目标邮箱 \|.*$/m,()=>`| 已分配目标邮箱 | \`${s.recoveryEmail}\`（已绑定为恢复邮箱；AWS 主邮箱未变） |`);
      text = text.slice(0, a) + part + text.slice(b);
    }
  }
  fs.writeFileSync(file + '.tmp', text, {mode:0o600}); fs.chmodSync(file + '.tmp', 0o600); fs.renameSync(file + '.tmp', file);
}
async function waitHuman(page, {until, notify, timeout = 900000}) {
  await page.bringToFront(); notify();
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const text = await body(page); stage(text, page.url()); // Destination guard only; no credential writes.
    if (until(text, page.url())) return;
    await sleep(1000);
  }
  throw new Error('Human credential entry not completed; resume this browser');
}
async function enrollMfa(page, {secret, getTotp, beforeSubmit = () => {}}) {
  const text = await body(page), current = stage(text,page.url());
  if (current === 'mfa_registered' || current === 'security') return;
  if (current !== 'mfa_setup') throw new Error('Expected MFA setup page');
  const shown = [...new Set(text.match(/\b[A-Z2-7]{32,128}\b/g) || [])];
  if (shown.length !== 1 || shown[0] !== secret) throw new Error('Displayed MFA seed differs from saved checkpoint; no submission');
  const code = await getTotp();
  if (!/^\d{6}$/.test(code)) throw new Error('Expected six-digit TOTP');
  await page.getByRole('textbox',{name:/Authenticator code|身份验证器代码/i}).fill(code);
  await beforeSubmit();
  await page.getByRole('button',{name:/^Assign MFA$|^分配 MFA$/}).click();
  const deadline = Date.now()+30000;
  while(Date.now()<deadline) {
    const next = await body(page);
    if (stage(next,page.url()) === 'mfa_registered') return;
    if (/invalid|incorrect|expired|ERR-\d+|无效|错误|过期/i.test(next)) throw new Error('MFA enrollment rejected; browser retained, seed preserved');
    await sleep(500);
  }
  throw new Error('MFA enrollment result uncertain; inspect existing page before retrying');
}
async function changePassword(page, password, beforeSubmit = () => {}) {
  const success = (text,url) => url.includes('changePasswordSuccess=true') || /Your password has been successfully updated|您的密码已成功更新/.test(text);
  if(success(await body(page),page.url()))return;
  if(stage(await body(page),page.url())!=='new_password')throw new Error('Expected new-password form');
  const inputs=page.locator('input[type=password]:visible');
  await inputs.nth(1).waitFor(); // Heading renders before the controlled form inputs.
  if(await inputs.count()!==2)throw new Error('Expected exactly new and confirm password inputs');
  await inputs.nth(0).fill(password);await inputs.nth(1).fill(password);
  await beforeSubmit();
  await page.getByRole('button',{name:/^Change password$|^Save changes$|^更改密码$|^保存更改$/i}).click();
  const deadline=Date.now()+30000;
  while(Date.now()<deadline) {
    const text=await body(page);
    if(success(text,page.url()))return;
    stage(text,page.url());
    if(/ERR-\d+|password.*(?:incorrect|cannot|invalid)|密码.*(?:无效|错误)/i.test(text))throw new Error('Password change rejected; current password not updated in ledger');
    await sleep(500);
  }
  throw new Error('Password change result uncertain; inspect existing page before retrying');
}
async function main() {
  process.umask(0o077);
  const env = process.env, email = env.BUILDER_USER;
  const factorsOnly = env.AWS_SECURITY_SCOPE === 'factors';
  if(env.AWS_SECURITY_SCOPE && !['factors','full'].includes(env.AWS_SECURITY_SCOPE))throw new Error('Invalid AWS_SECURITY_SCOPE');
  if (process.argv.includes('--help')) { console.log('See kiro_builder_security.md. --check validates configuration. --otp prints a current code from MFA_SECRET_FILE for BUILDER_USER.'); return; }
  if (!email || !/^[A-Za-z0-9._+-]+@[A-Za-z0-9.-]+$/.test(email)) throw new Error('Invalid BUILDER_USER');
  if (process.argv.includes('--otp')) {
    const record = readSecrets(env.MFA_SECRET_FILE)[email]; if (!record?.secret) throw new Error('Saved MFA secret missing');
    const remaining=30-Math.floor(Date.now()/1000)%30;
    if(remaining<22)await sleep((remaining+0.1)*1000);
    const seconds=Math.floor(Date.now()/1000);
    console.log(JSON.stringify({code:totp(record.secret,seconds),validUntil:new Date((Math.floor(seconds/30)+1)*30000).toISOString()})); return;
  }
  for (const key of ['BUILDER_PASS','MAILBOX_PASSWORD','AWS_NEW_PASSWORD_FILE','MAIL_API_ADMIN_TOKEN_FILE','MIGRATION_STATE_FILE','MFA_SECRET_FILE','ACCOUNT_LEDGER_FILE','RECOVERY_EMAIL'])
    if (!env[key]) throw new Error('Missing ' + key);
  const targetPassword = fs.readFileSync(env.AWS_NEW_PASSWORD_FILE, 'utf8').trim();
  const token = fs.readFileSync(env.MAIL_API_ADMIN_TOKEN_FILE, 'utf8').trim();
  const stateAll = read(env.MIGRATION_STATE_FILE), record = stateAll[email] ||= {};
  const s = record.security ||= {status:'not_started'};
  if (record.alias?.alias_email !== env.RECOVERY_EMAIL) throw new Error('Recovery email must match this account dedicated alias receipt');
  if (!targetPassword || !token) throw new Error('Empty password/token file');
  if (!fs.readFileSync(env.ACCOUNT_LEDGER_FILE,'utf8').includes(email)) throw new Error('Account missing from ledger');
  if (process.argv.includes('--check')) { console.log('CONFIG_OK: no network or account mutations'); return; }
  if (s.status==='hardened_and_relogin_verified' || (factorsOnly&&s.status==='factors_and_relogin_verified')) { console.log('ALREADY_VERIFIED: '+s.freshLoginVerifiedAt+'; no repeated changes or new session'); return; }
  const lock = env.MIGRATION_STATE_FILE + '.security.lock';
  const lockFd = fs.openSync(lock, 'wx', 0o600); fs.writeFileSync(lockFd, String(process.pid));
  let browser, page, mail, baseline;
  const secret = () => readSecrets(env.MFA_SECRET_FILE)[email]?.secret;
  const currentPassword = () => s.passwordChangeObservedAt || s.passwordStatus==='shared_password_relogin_verified' ? targetPassword : env.BUILDER_PASS;
  const persist = () => {
    s.updatedAt = now(); const current = read(env.MIGRATION_STATE_FILE);
    current[email] = {...current[email],security:s}; save(env.MIGRATION_STATE_FILE,current);
    ledger(env.ACCOUNT_LEDGER_FILE,email,s,s.passwordUnchanged?currentPassword():targetPassword,secret());
  };
  const getTotp = async () => {
    if (!secret()) throw new Error('MFA secret unavailable');
    const remaining = 30 - Math.floor(Date.now()/1000)%30;
    if (remaining < 15) await sleep((remaining + 0.1)*1000);
    return totp(secret());
  };
  const mailCode = async () => {
    if (!mail) throw new Error('Old mailbox OTP needed; restart without AWS_CDP_URL to initialize Outlook');
    return oldEmailCode(mail, baseline);
  };
  const authTo = (password, done, overrides = {}) => auth(page,{email,password,getMailCode:mailCode,getTotp:secret()?getTotp:null,done,...overrides});
  const getRecoveryCode = async () => {
    const deadline=Date.now()+300000;
    while(Date.now()<deadline) {
      const data=await mailbox(env.MAIL_API_BASE || 'https://mail-api.example.invalid',token,'/mailboxes/'+encodeURIComponent(env.RECOVERY_EMAIL)+'/messages?limit=20');
      const code=freshMailCode(data.messages || [],s.recoveryCodeSince,env.RECOVERY_EMAIL);
      if(code)return code; await sleep(3000);
    }
    throw new Error('Fresh recovery-email OTP not received');
  };
  const security = async password => {
    await page.goto(SECURITY,{waitUntil:'domcontentloaded',timeout:60000}); await authTo(password,['security']);
    await page.getByText(/Signed in devices|登录设备/).first().waitFor();
    if(s.mfaEnrollmentSubmittedAt || s.mfaStatus?.startsWith('registered'))await page.getByText('TOTP',{exact:true}).first().waitFor();
    if(s.recoveryEmailStatus==='verified_in_security_page')await page.getByText(env.RECOVERY_EMAIL,{exact:true}).first().waitFor();
    return securitySnapshot(await body(page));
  };
  const freshLogin = async (password=targetPassword,unchanged=false) => {
    const context = await browser.newContext({locale:'en-US'}); page = await context.newPage();
    await page.goto(PROFILE,{waitUntil:'domcontentloaded',timeout:60000});
    const stages = new Set();
    await authTo(password,['profile'],{notify:event=>{stages.add(event);console.log(event);}});
    if(!stages.has('AUTH_STAGE: password') || !stages.has('AUTH_STAGE: mfa_login'))
      throw new Error('Fresh login did not exercise both password and MFA; not marked verified');
    const text = await body(page);
    if (!text.toLowerCase().includes(email.toLowerCase())) throw new Error('Fresh-login identity mismatch');
    const snap = await security(password);
    if (snap.recoveryEmail !== env.RECOVERY_EMAIL || !snap.hasTotp) throw new Error('Fresh-login security settings mismatch');
    s.freshLoginVerifiedAt = now(); s.passwordStatus = unchanged?'existing_password_relogin_verified':'shared_password_relogin_verified'; s.passwordUnchanged=unchanged; s.mfaStatus = 'registered_and_fresh_login_verified';
    storeSecret(env.MFA_SECRET_FILE,email,{...readSecrets(env.MFA_SECRET_FILE)[email],status:s.mfaStatus,freshLoginVerifiedAt:s.freshLoginVerifiedAt});
    persist(); return snap;
  };
  try {
    if (env.AWS_DEBUG_PORT && !/^\d{4,5}$/.test(env.AWS_DEBUG_PORT)) throw new Error('Invalid debug port');
    browser = env.AWS_CDP_URL ? await chromium.connectOverCDP(env.AWS_CDP_URL) : await chromium.launch({channel:'chrome',headless:false,slowMo:Number(env.KIRO_SLOW_MO_MS||0),
      proxy:{server:env.OAUTH_PROXY || 'http://127.0.0.1:7890'}, args:env.AWS_DEBUG_PORT?[`--remote-debugging-port=${env.AWS_DEBUG_PORT}`]:[]});
    const resumeStatus = s.status;
    if (env.AWS_CDP_URL) {
      const pending = ['recovery_email_requested','mfa_registration_open','awaiting_mfa_registration','awaiting_password_change'].includes(resumeStatus);
      const candidates=browser.contexts().flatMap(c=>c.pages()).filter(p=>pending
        ? new URL(p.url()).hostname==='us-east-1.sso.signin.aws' || p.url().startsWith('https://us-east-1.credentials.signin.aws/')
        : p.url().startsWith(PROFILE) || p.url().startsWith('https://us-east-1.credentials.signin.aws/'));
      if(candidates.length>1)throw new Error('Multiple AWS tabs in resume browser; leave only the intended account tab');
      page=candidates[0];
    }
    if (!page) {
      const context = await browser.newContext({locale:'en-US'}); mail = await context.newPage();
      await outlookLogin(mail,email,env.MAILBOX_PASSWORD,'https://outlook.office.com/mail/'); baseline = await outlookCodes(mail);
      page = await context.newPage();
    }
    if (!['recovery_email_requested','mfa_registration_open','awaiting_mfa_registration','awaiting_password_change'].includes(resumeStatus)) {
      await page.goto(PROFILE,{waitUntil:'domcontentloaded',timeout:60000});
      await authTo(currentPassword(),['profile']);
      if (!(await body(page)).toLowerCase().includes(email.toLowerCase())) throw new Error('Profile identity mismatch');
      s.identityVerifiedAt = now(); persist();
    } else if (!env.AWS_CDP_URL) {
      throw new Error('Pending setup step: resume the same browser with AWS_CDP_URL; no duplicate enrollment/change');
    }
    let snap;
    if(resumeStatus==='recovery_email_requested') {
      await authTo(env.BUILDER_PASS,['security'],{getMailCode:getRecoveryCode,transitionFrom:'recovery_form'});
      snap=securitySnapshot(await body(page));
      if(snap.recoveryEmail!==env.RECOVERY_EMAIL)throw new Error('Pending recovery email not verified');
      s.status='recovery_email_verified';persist();
    }
    if (resumeStatus === 'awaiting_mfa_registration') {
      if ((!page.url().includes('/mfa/register') && stage(await body(page),page.url())!=='security' && !s.mfaRegistrationObservedAt) || !s.identityVerifiedAt) throw new Error('Expected pending MFA page missing');
    } else if (!['awaiting_password_change','mfa_registration_open'].includes(resumeStatus)) {
      snap = snap || await security(currentPassword());
      if (snap.recoveryEmail && snap.recoveryEmail !== env.RECOVERY_EMAIL) throw new Error('Different recovery email already bound; no replacement');
      if (!snap.recoveryEmail) {
        s.status='recovery_email_requested'; s.recoveryCodeSince=now(); persist();
        await page.getByRole('button',{name:'add-recovery-email-button',exact:true}).click();
        await authTo(env.BUILDER_PASS,['recovery_form']);
        await page.getByRole('textbox',{name:'Recovery email',exact:true}).fill(env.RECOVERY_EMAIL);
        await page.getByRole('button',{name:/^Continue$|^继续$/}).click();
        await authTo(env.BUILDER_PASS,['security'],{getMailCode:getRecoveryCode,transitionFrom:'recovery_form'});
        snap=securitySnapshot(await body(page));
        if(snap.recoveryEmail!==env.RECOVERY_EMAIL)throw new Error('Recovery email not verified on security page');
      }
      s.recoveryEmail=env.RECOVERY_EMAIL;s.recoveryEmailStatus='verified_in_security_page';s.status='recovery_email_verified';persist();
      if (!snap.hasTotp) {
        if (secret()) throw new Error('Saved MFA secret exists but device missing; no silent overwrite');
        s.status='mfa_registration_open';persist();
        await page.getByRole('button',{name:'show-register-device-modal',exact:true}).click();
      } else if (!secret()) throw new Error('Existing MFA device without local secret; user verification required');
    }
    if(s.status==='mfa_registration_open') {
      const at=await authTo(env.BUILDER_PASS,['mfa_choice','mfa_setup']);
      if(at==='mfa_choice') {
        await page.getByRole('radio',{name:/Authenticator app|身份验证器应用程序/}).check();
        await page.getByRole('button',{name:/^Next$|^下一步$/}).click();
        await authTo(env.BUILDER_PASS,['mfa_setup'],{transitionFrom:'mfa_choice'});
      }
        await page.getByRole('button',{name:/Show secret key|Show key|显示密钥/i}).click();
        const shown=[...new Set((await body(page)).match(/\b[A-Z2-7]{32,128}\b/g)||[])];
        if(shown.length!==1)throw new Error('MFA secret unavailable or ambiguous');
        storeSecret(env.MFA_SECRET_FILE,email,{secret:shown[0],status:'pending_registration',algorithm:'SHA1',digits:6,period:30,recordedAt:now()});
        s.status='awaiting_mfa_registration';s.mfaStatus='secret_recorded_pending_registration';persist();
        if(!fs.readFileSync(env.ACCOUNT_LEDGER_FILE,'utf8').includes('`'+shown[0]+'`'))throw new Error('MFA ledger read-back failed');
        console.log('MFA_SEED_CHECKPOINT_OK: primary, backup and private ledger verified; no seed logged');
    }
    if (s.status==='awaiting_mfa_registration') {
      const saved=readSecrets(env.MFA_SECRET_FILE)[email];
      if(!saved?.secret)throw new Error('Pending MFA seed missing');
      storeSecret(env.MFA_SECRET_FILE,email,saved); // Re-verify both durable copies on resume.
      persist();
      if(!fs.readFileSync(env.ACCOUNT_LEDGER_FILE,'utf8').includes('`'+saved.secret+'`'))throw new Error('MFA ledger read-back failed');
      if(s.mfaEnrollmentSubmittedAt && stage(await body(page),page.url())==='mfa_setup')
        throw new Error('Previous MFA submission uncertain; inspect before retry');
      if(!s.mfaRegistrationObservedAt)await enrollMfa(page,{secret:saved.secret,getTotp,beforeSubmit:()=>{s.mfaEnrollmentSubmittedAt=now();persist();}});
      if(stage(await body(page),page.url())==='mfa_registered') {
        s.mfaRegistrationObservedAt=now();s.mfaStatus='registered_pending_security_page_check';persist();
        storeSecret(env.MFA_SECRET_FILE,email,{...saved,status:s.mfaStatus});
        await page.getByRole('button',{name:/^Done$|^完成$/}).click();
      }
      await authTo(env.BUILDER_PASS,['security'],{transitionFrom:'mfa_registered'});
      await page.getByText('TOTP',{exact:true}).first().waitFor();snap=securitySnapshot(await body(page));
      if(!snap.hasTotp)throw new Error('MFA registration not visible on security page');
      storeSecret(env.MFA_SECRET_FILE,email,{...readSecrets(env.MFA_SECRET_FILE)[email],status:'registered_verified_in_security_page'});
      s.mfaStatus='registered_verified_in_security_page';s.status='mfa_registered';persist();
    }
    if(factorsOnly){
      snap=await freshLogin(currentPassword(),true);
      s.activeSessionCountAfterVerification=snap.sessions;s.sessionsRevokeStatus='not_requested';s.status='factors_and_relogin_verified';delete s.lastFailure;persist();
      console.log('SECURITY_FACTORS_VERIFIED: recovery email and MFA checked in a fresh login; passwords unchanged; no session deletion');return;
    }
    if (!s.passwordChangeObservedAt && s.passwordStatus!=='shared_password_relogin_verified') {
      if (s.status!=='awaiting_password_change') {
        delete s.passwordUnchanged;
        await page.getByRole('button',{name:'change-password-button',exact:true}).click();
        await authTo(env.BUILDER_PASS,['new_password']);
        s.status='awaiting_password_change';s.passwordStatus='shared_password_prepared';persist();
      }
      if(s.passwordChangeSubmittedAt && stage(await body(page),page.url())==='new_password')
        throw new Error('Previous password submission uncertain; inspect before retry');
      await changePassword(page,targetPassword,()=>{s.passwordChangeSubmittedAt=now();persist();});
      s.passwordChangeObservedAt=now();s.passwordStatus='success_banner_seen_pending_login_verification';s.status='password_change_observed';persist();
    }
    snap=await freshLogin();
    if(!s.sessionsRevokeSubmittedAt) {
      await page.getByRole('button',{name:'show-Delete all active sessions-modal',exact:true}).click();
      await page.getByRole('textbox',{name:/delete all/i}).fill('delete all');
      s.sessionsRevokeSubmittedAt=now();s.sessionsRevokeStatus='submitted_result_pending';persist();
      await page.getByRole('button',{name:'confirm-delete-all-sessions',exact:true}).click();
      await page.waitForURL(url=>new URL(url).hostname==='us-east-1.sso.signin.aws',{timeout:30000});
      snap=await freshLogin();
    }
    s.activeSessionCountAfterVerification=snap.sessions;
    s.sessionsRevokeStatus=snap.sessions===1?'old_devices_absent_after_relogin':'propagation_pending_no_automatic_repeat';
    s.status=snap.sessions===1?'hardened_and_relogin_verified':'session_revocation_pending';
    if(snap.sessions===1)delete s.lastFailure;
    persist();
    if(snap.sessions!==1)throw new Error('Old sessions still listed; AWS propagation may take one hour');
    console.log('SECURITY_VERIFIED: recovery email, MFA, shared password and one fresh session; no Prism import performed');
  } catch(error) {
    // Deliberately no Playwright call log, form contents, secrets or raw page dumps.
    s.lastFailure=String(error.message || error).split('\n')[0];
    for(const value of [env.BUILDER_PASS,env.MAILBOX_PASSWORD,targetPassword,token,secret()])if(value)s.lastFailure=s.lastFailure.split(value).join('[redacted]');
    persist();console.error('SECURITY_STOPPED: '+s.lastFailure);process.exitCode=1;
    if(page && !page.isClosed())await page.bringToFront().catch(()=>{});
  } finally {
    fs.closeSync(lockFd);fs.unlinkSync(lock);
    if(browser?.isConnected()) {
      if(env.AWS_CDP_URL)await browser.close(); // disconnect only
      else if(env.KEEP_OPEN==='0'||(env.KEEP_OPEN==='on-error'&&['hardened_and_relogin_verified','factors_and_relogin_verified'].includes(s.status)))await browser.close();
      else {console.log('BROWSER_KEPT_OPEN: close Chrome to exit, or resume using AWS_CDP_URL');await new Promise(r=>browser.once('disconnected',r));}
    }
  }
}
if(require.main===module)main().catch(()=>{console.error('SECURITY_FAILED: check configuration/lock file');process.exitCode=1;});
module.exports={totp,freshMailCode,stage,securitySnapshot,ledger,waitHuman,auth,readSecrets,storeSecret,enrollMfa,changePassword};
