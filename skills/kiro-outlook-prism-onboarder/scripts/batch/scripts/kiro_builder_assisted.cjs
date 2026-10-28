#!/usr/bin/env node
// Headed, sequential Builder ID login. CAPTCHA is always completed by the user.
const { chromium } = require('playwright');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { spawn, spawnSync } = require('node:child_process');
const { promisify } = require('node:util');
const execFile = promisify(require('node:child_process').execFile);
const {totp,readSecrets} = require('./kiro_builder_security.cjs');
const {createEmailRetry} = require('./kiro_email_retry.cjs');

process.umask(0o077);
const runDir = process.env.KIRO_RUN_DIR || path.join(os.homedir(), '.local/state/kiro-onboarding/kiro-builder');
fs.mkdirSync(runDir, { recursive: true, mode: 0o700 });
const env = { ...process.env, KIRO_RUN_DIR: runDir };
const backend = path.join(__dirname, 'kiro_builder_backend.py');
const python = process.env.PYTHON || 'python3';
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
const log = (event, email, detail = '') => console.log(JSON.stringify({ at: new Date().toISOString(), event, email, detail }));
const text = page => page.locator('body').innerText({ timeout: 5000 }).catch(() => '');
const visible = async locator => {
  for (const item of await locator.all()) if (await item.isVisible()) return item;
  return null;
};
async function click(page, name) {
  const item = await visible(page.getByRole('button', { name }));
  if (!item || !await item.isEnabled()) return false;
  await item.click();
  return true;
}
async function fill(page, selector, value) {
  const item = await visible(page.locator(selector));
  if (!item) return false;
  await item.fill(value);
  if (await item.inputValue() !== value) throw new Error('Input did not match supplied value');
  return true;
}
async function go(page, url) {
  await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 60000 });
}
async function backendRun(...args) {
  const { stdout } = await execFile(python, [backend, ...args], { env, timeout: 600000, maxBuffer: 1024 * 1024 });
  return stdout;
}
function receipts() {
  const file = path.join(runDir, 'verified-imports.json');
  return fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, 'utf8')).accounts : [];
}
function readAccounts(file) {
  return fs.readFileSync(file, 'utf8').split(/\r?\n/).filter(l => l.trim() && !l.startsWith('#')).map((line, i) => {
    const fields = line.split('\t');
    if (![3, 6].includes(fields.length) || !fields[0].includes('@') || fields.some(x => !x))
      throw new Error(`Line ${i + 1}: expected email<TAB>mailPassword<TAB>awsPassword or external OTP fields`);
    return { email: fields[0].replace(/\\@/g, '@'), mailPassword: fields[1], awsPassword: fields[2],
      otpClientId: fields[3], otpToken: fields[4], otpUrl: fields[5] };
  });
}
function extractCodes(body) {
  return [...body.matchAll(/Verification code\s+(\d{6})(?!\d)/gi)].map(m => m[1]);
}
function awsAccountError(body) {
  return /\bERR-837\b/i.test(body);
}
async function mailLogin(page, account) {
  await go(page, 'https://outlook.office.com/mail/');
  let emailSent = false, passwordSent = false;
  let lastStage = '';
  let lastSnapshot = Date.now();
  const deadline = Date.now() + 180000;
  while (Date.now() < deadline) {
    const body = await text(page);
    const stage = /Stay signed in/.test(body) ? 'stay-signed-in' :
      /Enter password/.test(body) ? 'password' :
      /outlook\.office\.com\/mail/.test(page.url()) ? 'inbox-loading' : 'email-or-redirect';
    if (stage !== lastStage) { log('MAIL_STAGE', account.email, stage); lastStage = stage; }
    if (Date.now() - lastSnapshot > 20000 && stage !== 'inbox-loading') {
      await page.screenshot({ path: path.join(runDir, 'mail-progress.png') });
      lastSnapshot = Date.now();
    }
    if (/outlook\.office\.com\/mail/.test(page.url()) &&
        await page.locator('[role="listbox"]').count()) {
      if (!body.toLowerCase().includes(account.email.toLowerCase())) { await pause(1500); continue; }
      log('MAIL_READY', account.email);
      return new Set(extractCodes(body));
    }
    if (/password is incorrect|incorrect.*password|account.*locked|incorrect username|couldn.t find an account/i.test(body))
      throw new Error('Outlook rejected the supplied account/password');
    if (/Stay signed in\?/i.test(body)) await click(page, /^No$/i);
    // Microsoft's email page contains a password autofill decoy that is technically visible.
    // Gate on the actual page heading, not isVisible() alone.
    else if (stage === 'password' && !passwordSent && await fill(page, 'input[type="password"]:not([aria-hidden="true"])', account.mailPassword)) {
      if (await click(page, /^Sign in$/i)) { passwordSent = true; log('MAIL_PASSWORD_SENT', account.email); }
    } else if (stage !== 'password' && !emailSent && await fill(page, 'input[type="email"]:not([aria-hidden="true"]), input[name="loginfmt"]:not([aria-hidden="true"])', account.email)) {
      if (await click(page, /^Next$/i)) { emailSent = true; log('MAIL_EMAIL_SENT', account.email); }
    } else if (emailSent && /issue looking up your account/i.test(body)) {
      throw new Error('Outlook account lookup failed; retry the run after checking connectivity');
    }
    await pause(1500);
  }
  await page.screenshot({ path: path.join(runDir, 'attention.png') });
  throw new Error(`Outlook login requires attention (${lastStage}); see attention.png`);
}
async function newestCode(page, baseline) {
  await page.reload({ waitUntil: 'domcontentloaded', timeout: 60000 });
  const deadline = Date.now() + 120000;
  let refreshed = Date.now();
  while (Date.now() < deadline) {
    const body = await text(page);
    const code = extractCodes(body).find(c => !baseline.has(c));
    if (code) return code;
    // Some Outlook layouts omit the code from the preview; open only the newest AWS message.
    const row = page.getByRole('option').filter({ hasText: /no-reply@login\.awsapps\.com/ }).first();
    if (await row.isVisible().catch(() => false)) await row.click();
    if (Date.now() - refreshed > 15000) {
      await page.reload({ waitUntil: 'domcontentloaded', timeout: 60000 });
      refreshed = Date.now();
    }
    await pause(2000);
  }
  throw new Error('No new AWS verification code arrived; old codes were not reused');
}
async function externalCode(account, baseline) {
  const deadline = Date.now() + 120000;
  while (Date.now() < deadline) {
    const response = await fetch(account.otpUrl, { method:'POST', headers:{'Content-Type':'application/json','Origin':new URL(account.otpUrl).origin,'Referer':account.otpUrl},
      body: JSON.stringify({email:account.email, client_id:account.otpClientId, token:account.otpToken, limit:10}) });
    if (!response.ok) throw new Error(`External OTP endpoint HTTP ${response.status}`);
    const data = await response.json();
    const messages = data.emails || [];
    const candidates = messages.filter(m => Date.parse(m.received_at || m.date || 0) >= baseline.at)
      .sort((a,b)=>Date.parse(b.received_at || b.date || 0)-Date.parse(a.received_at || a.date || 0));
    for (const m of candidates) {
      const body = String(m.body_text || m.body || m.content || '') + ' ' + String(m.subject || '');
      const code = body.match(/\\b(\\d{6})\\b/);
      if (code) return code[1];
    }
    await pause(2000);
  }
  throw new Error('No new AWS verification code arrived from external OTP endpoint');
}
let activeDevice;
async function startDevice() {
  let output = '', done = false, status;
  const child = spawn(python, [backend, 'login'], { env, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
  activeDevice = child;
  child.stdout.on('data', chunk => { output += chunk.toString(); });
  child.stderr.on('data', () => {});
  child.on('error', () => { done = true; status = -1; });
  child.on('close', code => { done = true; status = code; activeDevice = null; });
  const deadline = Date.now() + 60000;
  while (Date.now() < deadline && !done) {
    const match = output.match(/Code:\s*([A-Z]{4}-[A-Z]{4})/);
    if (match) return { child, code: match[1], state: () => ({ done, status }) };
    await pause(300);
  }
  child.kill();
  throw new Error('Kiro CLI did not produce a device code');
}
async function submitSavedMfa(page, account) {
  if (!account.mfaSecret) throw new Error('AWS requests MFA but no saved secret is configured');
  const u=new URL(page.url());
  if (u.protocol!=='https:' || !['us-east-1.sso.signin.aws','us-east-1.signin.aws'].includes(u.hostname) || u.pathname.includes('/mfa/register'))
    throw new Error('Unexpected MFA destination; refusing to submit code');
  const remaining=30-Math.floor(Date.now()/1000)%30;
  if(remaining<15)await pause((remaining+0.1)*1000);
  await page.getByRole('textbox',{name:/^MFA code$|^MFA 代码$/}).fill(totp(account.mfaSecret));
  if(await click(page,/^Continue$|^继续$/)) {
    log('AWS_MFA_SENT',account.email,'Used saved TOTP seed; code not logged');return true;
  }
  return false;
}
async function awsLogin(page, mail, account, baseline, device) {
  await go(page, `https://view.awsapps.com/start/#/device?user_code=${device.code}`);
  let emailSent = false, passwordSent = false, otpSent = false, mfaSent = false, notified = false;
  const emailRetry=createEmailRetry({notify:detail=>log('AWS_EMAIL_RETRY',account.email,detail)});
  const deadline = Date.now() + 15 * 60000;
  while (Date.now() < deadline + emailRetry.pausedMs()) {
    const state = device.state();
    if (state.done) {
      if (state.status !== 0) throw new Error('Kiro CLI login expired or lost connectivity');
      return;
    }
    const body = await text(page);
    if(await emailRetry.handle(page,body,account.email))continue;
    if (awsAccountError(body)) {
      const screenshot = path.join(runDir, `${account.email}-aws-account-error-${Date.now()}.png`);
      await page.screenshot({ path: screenshot, fullPage: true });
      const detail = { at: new Date().toISOString(), email: account.email, error: 'ERR-837 Account issue',
        requestId: body.match(/Request ID:\s*([^\s]+)/)?.[1], screenshot };
      fs.appendFileSync(path.join(runDir, 'login-failures.jsonl'), JSON.stringify(detail) + '\n');
      const closed = new Promise(resolve => device.child.once('close', resolve));
      process.kill(-device.child.pid, 'SIGTERM');
      await closed;
      const error = new Error('AWS ERR-837 Account issue; evidence saved');
      error.skipAccount = true;
      throw error;
    }
    if (/Request approved|请求已批准/.test(body)) {
      await pause(1000);
      continue;
    }
    if (/Enter your name|Create your password/.test(body)) {
      try { process.kill(-device.child.pid, 'SIGTERM'); } catch {}
      throw new Error('AWS opened signup instead of existing-account login; inspect the visible page');
    }
    if (/session has expired|no longer valid|information.*doesn.t (?:quite )?match|信息.*不.*相符|invalid.*code|incorrect.*password/i.test(body))
      throw new Error('AWS rejected the login or code; inspect the visible page');
    // Snapshot the page stage before awaiting locators: navigation may complete between awaits.
    const onPasswordPage = /Sign in with your AWS Builder ID/.test(body);
    const password = onPasswordPage ? await visible(page.locator('input[type="password"]')) : null;
    if (password && !body.includes(account.email)) { await pause(1000); continue; }
    if (password && !passwordSent && await password.inputValue() !== account.awsPassword)
      await password.fill(account.awsPassword);
    if (/Security check/.test(body) && !/successfully solved|Success!/i.test(body)) {
      if (!notified) {
        await page.bringToFront();
        log('HUMAN_REQUIRED', account.email, 'Complete Verify in Chrome; the script resumes automatically.');
        notified = true;
      }
      await pause(1500);
      continue;
    }
    notified = false;
    if (/Additional verification required|需要额外验证|MFA code|MFA 代码/.test(body)) {
      if (!mfaSent) mfaSent = await submitSavedMfa(page,account);
    } else if (/Verify your identity/.test(body)) {
      if (!body.includes(account.email)) throw new Error('AWS identity mismatch');
      if (!otpSent) {
        const code = account.otpToken ? await externalCode(account, baseline) : await newestCode(mail, baseline);
        await page.bringToFront();
        const input = page.getByLabel('Verification code', { exact: true });
        await input.fill(code);
        if (await click(page, /^Continue$/)) otpSent = true;
      }
    } else if (/Authorization requested/.test(body)) {
      if (!body.includes(device.code)) throw new Error('Device code mismatch');
      await click(page, /^Confirm and continue$/);
    } else if (/Allow Kiro CLI to access your data\?/.test(body)) {
      await click(page, /^Allow access$/);
    } else if (password && !passwordSent) {
      if (!body.includes(account.email)) throw new Error('AWS password page identity mismatch');
      if (await password.inputValue() !== account.awsPassword) throw new Error('AWS password input did not match supplied value');
      log('AWS_INPUT_VERIFIED', account.email, 'Password input exactly matches the supplied value');
      if (await click(page, /^Continue$/)) { passwordSent = true; log('AWS_PASSWORD_SENT', account.email); }
    } else if (!emailSent && await fill(page, 'input[type="email"]', account.email)) {
      if (await click(page, /^Continue$/)) { emailSent = true; emailRetry.submitted(); log('AWS_EMAIL_SENT', account.email); }
    }
    await pause(1500);
  }
  throw new Error('AWS login timed out; leave the page for inspection');
}

let browser;
async function main() {
  const file = process.argv[2];
  if (!file) throw new Error('Usage: node kiro_builder_assisted.cjs /private/accounts.tsv [--check]');
  const accounts = readAccounts(file);
  if (process.env.KIRO_MFA_SECRET_FILE) {
    const secrets = readSecrets(process.env.KIRO_MFA_SECRET_FILE);
    for (const account of accounts) {
      const record=secrets[account.email];
      if(!record?.secret || !record.status?.startsWith('registered'))throw new Error('Registered MFA seed missing for '+account.email);
      account.mfaSecret=record.secret;
    }
  }
  if (process.argv.includes('--check')) {
    console.log(JSON.stringify({ accounts: accounts.map(a => a.email), runDir }));
    return;
  }
  const lock = path.join(runDir, 'runner.lock');
  const fd = fs.openSync(lock, 'wx', 0o600);
  fs.writeFileSync(fd, String(process.pid));
  process.on('exit', () => {
    if (activeDevice) { try { process.kill(-activeDevice.pid, 'SIGTERM'); } catch {} }
    fs.closeSync(fd); fs.unlinkSync(lock);
  });
  process.on('SIGINT', () => process.exit(130));
  process.on('SIGTERM', () => process.exit(143));
  let identity = spawnSync('kiro-cli', ['whoami'], { encoding: 'utf8' }).stdout.match(/^Email:\s*(.+)$/m)?.[1];
  browser = await chromium.launch({ channel: 'chrome', headless: false,
    args: process.env.KIRO_CDP_PORT ? [`--remote-debugging-port=${Number(process.env.KIRO_CDP_PORT)}`] : [],
    proxy: { server: process.env.KIRO_PROXY || 'http://127.0.0.1:7890' } });
  for (const account of accounts) {
    if (receipts().some(r => r.email === account.email && r.verification?.success)) {
      log('SKIPPED_VERIFIED', account.email);
      continue;
    }
    log('START', account.email);
    if (identity && identity !== account.email) {
      await backendRun('detach', identity);
      identity = null;
    }
    if (identity !== account.email) {
      const context = await browser.newContext({ locale: 'en-US' });
      const mail = await context.newPage();
      const baseline = account.otpToken ? { at: Date.now() } : await mailLogin(mail, account);
      const device = await startDevice();
      const aws = await context.newPage();
      try {
        await awsLogin(aws, mail, account, baseline, device);
      } catch (error) {
        if (!error.skipAccount) throw error;
        log('ACCOUNT_SKIPPED', account.email, error.message);
        await context.close();
        continue;
      }
      log('AWS_AUTHORIZED', account.email);
      await context.close();
    }
    log('IMPORTING', account.email, 'Checking existing accounts, importing, and running one verification request');
    await backendRun('import', account.email);
    identity = account.email;
    const receipt = receipts().find(r => r.email === account.email);
    log('VERIFIED', account.email, `prism account #${receipt.id}`);
  }
  await browser.close();
  log('COMPLETE', '', `${accounts.length} account rows processed`);
}
if (require.main === module) main().catch(error => {
  log('NEEDS_ATTENTION', '', error.message);
  // Keep the browser available for inspection. Stop with Ctrl+C, then rerun the same TSV.
  if (browser?.isConnected()) setInterval(() => {}, 60000);
  else process.exitCode = 1;
});
module.exports = { extractCodes, readAccounts, awsAccountError, awsLogin, submitSavedMfa, startDevice };
