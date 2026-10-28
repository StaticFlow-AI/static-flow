#!/usr/bin/env node
// AWS Builder ID email migration. No passwords, tokens or OTPs in receipts.
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require('playwright');
const { login: outlookLogin } = require('./outlook_change_password.cjs');
const { createEmailRetry } = require('./kiro_email_retry.cjs');
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
const PROFILE = 'https://profile.aws.amazon.com/';

function save(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true, mode: 0o700 });
  fs.writeFileSync(file + '.tmp', JSON.stringify(value, null, 2) + '\n', { mode: 0o600 });
  fs.chmodSync(file + '.tmp', 0o600);
  fs.renameSync(file + '.tmp', file);
}
function apiCode(messages, since, recipient) {
  const candidates = messages.filter(m =>
    /^(?:.*<)?no-reply@signin\.aws>?$/i.test(m.sender || '') &&
    Date.parse(m.received_at) >= Date.parse(since) &&
    (!m.recipient || m.recipient.toLowerCase().includes(recipient.toLowerCase())) &&
    /Verify your AWS Builder ID email address/i.test(m.subject || '')
  ).sort((a, b) => Date.parse(b.received_at) - Date.parse(a.received_at));
  if (!candidates.length) return null;
  const m = candidates[0];
  const text = (m.body_text || '') + ' ' + (m.body_html || '')
    .replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, ' ').replace(/<[^>]+>/g, ' ')
    .replace(/&nbsp;|&#160;/g, ' ');
  const codes = [...new Set(text.match(/\b\d{6}\b/g) || [])];
  return codes.length === 1 ? codes[0] : null;
}
function trustedAWS(url) {
  const u = new URL(url);
  return u.protocol === 'https:' && ['profile.aws.amazon.com', 'us-east-1.sso.signin.aws'].includes(u.hostname);
}
function outcome(current, original, target) {
  if (current.toLowerCase() === target.toLowerCase()) return 'profile_email_changed';
  if (current.toLowerCase() === original.toLowerCase()) return 'email_change_failed';
  throw new Error('Unexpected AWS profile identity');
}
function updateLedger(file, email, state) {
  if (!file) return;
  let text = fs.readFileSync(file, 'utf8');
  if (!text.includes(email)) throw new Error('Account missing from ledger');
  const start = `<!-- AWS_EMAIL_MIGRATION:${email}:START -->`;
  const end = `<!-- AWS_EMAIL_MIGRATION:${email}:END -->`;
  const statuses = {
    profile_email_changed: 'AWS 资料页已确认换绑成功；尚未验证新邮箱重新登录／Kiro 导入',
    email_change_failed: 'AWS 换绑失败；刷新资料页仍为原邮箱',
    'email_change_failed_ERR-837': 'AWS 换绑失败（ERR-837）；刷新资料页仍为原邮箱',
    verification_submitted: '已提交换绑验证，结果待核对；不得自动重复提交',
    verification_requested: '已请求新邮箱验证码，尚未确认换绑',
    alias_allocated: '已分配新邮箱，尚未换绑',
  };
  const block = `${start}\n### AWS 邮箱迁移状态：${email}\n\n` +
    `- 状态：${statuses[state.status] || '待处理'}。\n` +
    `- 原 AWS 邮箱：\`${email}\`。\n` +
    `- 目标邮箱：\`${state.alias?.alias_email || '未分配'}\`。\n` +
    `- 最后刷新确认的 AWS 邮箱：\`${state.currentEmail || '尚未读取'}\`。\n` +
    `- 最近 AWS 错误：${state.errorCode || '无已记录错误码'}；验证接口 HTTP 状态：${state.httpStatus || '未记录'}。\n` +
    `- 更新时间（UTC）：${state.checkedAt || state.updatedAt}。密码未由此脚本更改；此状态不代表已导入 prism01。\n${end}`;
  const a = text.indexOf(start), b = text.indexOf(end);
  if ((a === -1) !== (b === -1) || (a !== -1 && b < a)) throw new Error('Invalid ledger migration markers');
  text = a === -1 ? text.trimEnd() + '\n\n' + block + '\n' : text.slice(0, a) + block + text.slice(b + end.length);
  fs.writeFileSync(file + '.tmp', text, { mode: 0o600 });
  fs.chmodSync(file + '.tmp', 0o600);
  fs.renameSync(file + '.tmp', file);
}
async function mailbox(base, token, route, data) {
  const url = new URL(route, base);
  if (url.protocol !== 'https:') throw new Error('Mailbox API requires HTTPS');
  const response = await fetch(url, {
    method: data ? 'POST' : 'GET', redirect: 'error', signal: AbortSignal.timeout(30000),
    headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' },
    ...(data ? { body: JSON.stringify(data) } : {}),
  });
  if (!response.ok) throw new Error(`Mailbox API HTTP ${response.status}`);
  return response.json();
}
async function profileEmail(page) {
  if (new URL(page.url()).hostname !== 'profile.aws.amazon.com') throw new Error('Not on AWS profile');
  await page.goto(PROFILE + '#/profile/details', { waitUntil: 'domcontentloaded' });
  // Read server-backed state, not the optimistic UI after verification.
  await page.reload({ waitUntil: 'domcontentloaded' });
  await page.getByText('Contact information', { exact: true }).waitFor({ timeout: 30000 });
  const body = await page.locator('body').innerText();
  const match = body.match(/Contact information\s+Edit\s+Email address\s+([^\s]+@[^\s]+)/);
  if (!match) throw new Error('AWS profile email unavailable');
  return match[1];
}
async function outlookCodes(page) {
  const rows = page.getByRole('option').filter({ hasText: 'no-reply@login.awsapps.com' });
  const codes = new Set();
  for (const row of (await rows.all()).slice(0, 10)) {
    for (const m of (await row.innerText()).matchAll(/Verification code\s+(\d{6})(?!\d)/gi)) codes.add(m[1]);
  }
  return codes;
}
async function oldEmailCode(page, baseline) {
  const deadline = Date.now() + 300000;
  while (Date.now() < deadline) {
    await page.reload({ waitUntil: 'domcontentloaded' });
    await page.getByRole('listbox').first().waitFor({ timeout: 30000 });
    const codes = await outlookCodes(page);
    const code = [...codes].find(c => !baseline.has(c));
    if (code) { baseline.add(code); return code; }
    await sleep(4000);
  }
  throw new Error('No fresh AWS login code in Outlook message preview');
}
async function awsLogin(page, email, password, getCode, timeout) {
  await page.goto(PROFILE, { waitUntil: 'domcontentloaded', timeout: 60000 });
  let emailSent = false, passwordSent = false, otpSent = false, human = false;
  const emailRetry=createEmailRetry();
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline + emailRetry.pausedMs()) {
    if (!trustedAWS(page.url())) throw new Error('Unexpected AWS login destination');
    const body = await page.locator('body').innerText();
    if (new URL(page.url()).hostname === 'profile.aws.amazon.com' && /Contact information/.test(body)) return;
    if (/Enter your name|Create your password/.test(body)) throw new Error('AWS opened signup; stopped');
    if(await emailRetry.handle(page,body,email))continue;
    if (/incorrect.*password|invalid.*code|code.*expired|ERR-\d+/i.test(body)) {
      const error = new Error('AWS login rejected; inspect browser');
      error.awsCode = body.match(/ERR-\d+/)?.[0];
      error.requestId = body.match(/Request ID:\s*([a-z0-9-]+)/i)?.[1];
      throw error;
    }
    if (/Verify your identity/.test(body)) {
      if (!body.toLowerCase().includes(email.toLowerCase())) throw new Error('AWS login identity mismatch');
      if (!otpSent) {
        await page.getByRole('textbox', { name: /Verification code/i }).fill(await getCode());
        await page.getByRole('button', { name: 'Continue', exact: true }).click();
        otpSent = true;
      }
    } else {
      const pass = page.locator('input[type=password]');
      if (await pass.isVisible().catch(() => false)) {
        if (!body.toLowerCase().includes(email.toLowerCase())) throw new Error('AWS password identity mismatch');
        if (!passwordSent && await pass.inputValue() !== password) await pass.fill(password);
      }
      if (/Security check|Security Verification|Invalid captcha/i.test(body) && !/successfully solved|Success!/i.test(body)) {
        if (!human) { console.log('HUMAN_REQUIRED: complete CAPTCHA; script will resume automatically'); await page.bringToFront(); human = true; }
        await sleep(1500); continue;
      }
      human = false;
      if (await pass.isVisible().catch(() => false)) {
        if (!passwordSent) { await page.getByRole('button', { name: 'Continue', exact: true }).click(); passwordSent = true; }
      } else if (!emailSent) {
        const input = page.getByRole('textbox', { name: /email/i });
        if (await input.isVisible().catch(() => false)) {
          await input.fill(email); await page.getByRole('button', { name: 'Continue', exact: true }).click(); emailRetry.submitted(); emailSent = true;
        }
      }
    }
    await sleep(1000);
  }
  throw new Error('AWS login timed out; browser kept for inspection');
}
async function changeEmail(page, { email, state, persist, getCode }) {
  await page.getByRole('link', { name: 'Edit', exact: true }).nth(1).click();
  await page.getByRole('textbox', { name: /Email address/i }).fill(state.alias.alias_email);
  state.status = 'verification_requested'; state.verificationRequestedAt = new Date().toISOString(); persist();
  await page.getByRole('button', { name: 'Verify email', exact: true }).click();
  const field = page.getByRole('textbox', { name: /Verification code/i });
  await field.waitFor({ timeout: 30000 });
  await field.fill(await getCode());
  state.status = 'verification_submitted'; state.attempts = (state.attempts || 0) + 1; persist();
  const response = page.waitForResponse(r => new URL(r.url()).pathname === '/api/validateemailchange', { timeout: 30000 });
  // Attach a rejection handler before clicking; an interrupted click must not leak a rejected promise.
  const received = response.catch(() => null);
  await page.getByRole('button', { name: 'Verify', exact: true }).click();
  const result = await received;
  state.httpStatus = result?.status() || null;
  await page.getByText('Contact information', { exact: true }).waitFor({ timeout: 30000 });
  state.errorCode = (await page.locator('body').innerText()).match(/ERR-\d+/)?.[0] || null;
  state.currentEmail = await profileEmail(page);
  state.status = outcome(state.currentEmail, email, state.alias.alias_email);
  state.checkedAt = new Date().toISOString(); persist();
  return state.status;
}
async function main() {
  process.umask(0o077);
  if (process.argv.includes('--help')) {
    console.log('BUILDER_USER, BUILDER_PASS, MAILBOX_PASSWORD, MAIL_API_ADMIN_TOKEN_FILE, MIGRATION_STATE_FILE, ACCOUNT_LEDGER_FILE required. Optional AWS_CDP_URL to resume your browser. --retry allows one more attempt after a confirmed failure; --check validates configuration without network. See kiro_change_builder_email.md.'); return;
  }
  const env = process.env, email = env.BUILDER_USER;
  for (const key of ['BUILDER_USER','BUILDER_PASS','MAILBOX_PASSWORD','MAIL_API_ADMIN_TOKEN_FILE','MIGRATION_STATE_FILE','ACCOUNT_LEDGER_FILE'])
    if (!env[key]) throw new Error(`Missing ${key}`);
  if (!/^[a-zA-Z0-9._+-]+@[a-zA-Z0-9.-]+$/.test(email)) throw new Error('Invalid email');
  const token = fs.readFileSync(env.MAIL_API_ADMIN_TOKEN_FILE, 'utf8').trim();
  if (!token || /[\r\n]/.test(token)) throw new Error('Token file must contain only the admin token');
  if (env.AWS_DEBUG_PORT && (!/^\d+$/.test(env.AWS_DEBUG_PORT) || Number(env.AWS_DEBUG_PORT) < 1024 || Number(env.AWS_DEBUG_PORT) > 65535))
    throw new Error('AWS_DEBUG_PORT must be a local TCP port between 1024 and 65535');
  const all = fs.existsSync(env.MIGRATION_STATE_FILE) ? JSON.parse(fs.readFileSync(env.MIGRATION_STATE_FILE, 'utf8')) : {};
  const state = all[email] ||= { status: 'not_started' };
  if (!fs.readFileSync(env.ACCOUNT_LEDGER_FILE, 'utf8').includes(email)) throw new Error('Account missing from ledger');
  if (process.argv.includes('--check')) { console.log('CONFIG_OK: no network or account changes'); return; }
  let browser, page;
  const persist = () => { state.updatedAt = new Date().toISOString(); save(env.MIGRATION_STATE_FILE, all); updateLedger(env.ACCOUNT_LEDGER_FILE, email, state); };
  const base = env.MAIL_API_BASE || 'https://mail-api.example.invalid';
  const api = (route, data) => mailbox(base, token, route, data);
  const newCode = async () => {
    const deadline = Date.now() + 300000;
    while (Date.now() < deadline) {
      const data = await api('/mailboxes/' + encodeURIComponent(state.alias.alias_email) + '/messages?limit=20');
      const code = apiCode(data.messages || [], state.verificationRequestedAt, state.alias.alias_email);
      if (code) return code;
      await sleep(3000);
    }
    throw new Error('No fresh new-email verification code');
  };
  try {
    browser = env.AWS_CDP_URL ? await chromium.connectOverCDP(env.AWS_CDP_URL) : await chromium.launch({
      channel: 'chrome', headless: false, proxy: { server: env.OAUTH_PROXY || 'http://127.0.0.1:7890' },
      ...(env.AWS_DEBUG_PORT ? { args: [`--remote-debugging-port=${env.AWS_DEBUG_PORT}`] } : {}),
    });
    // Only attach to an explicit AWS Profile tab, never an unrelated account tab.
    page = env.AWS_CDP_URL ? browser.contexts().flatMap(c => c.pages()).find(p => p.url().startsWith(PROFILE)) : null;
    if (!page) {
      if (['verification_submitted','profile_email_changed'].includes(state.status))
        throw new Error('Resume an authenticated profile via AWS_CDP_URL to reconcile uncertain/completed change');
      const context = await browser.newContext({ locale: 'en-US' });
      const mail = await context.newPage();
      console.log('MIGRATION_STAGE: outlook-login');
      await outlookLogin(mail, email, env.MAILBOX_PASSWORD, 'https://outlook.office.com/mail/');
      const baseline = await outlookCodes(mail);
      page = await context.newPage();
      console.log('MIGRATION_STAGE: aws-login');
      await awsLogin(page, email, env.BUILDER_PASS, () => oldEmailCode(mail, baseline), Number(env.CAPTCHA_ASSIST_TIMEOUT || 600) * 1000);
    }
    state.currentEmail = await profileEmail(page);
    if (state.alias && outcome(state.currentEmail, email, state.alias.alias_email) === 'profile_email_changed') {
      state.status = 'profile_email_changed'; state.checkedAt = new Date().toISOString(); persist();
      console.log('PROFILE_EMAIL_CHANGED: reconciled, no duplicate submission'); return;
    }
    if (state.currentEmail.toLowerCase() !== email.toLowerCase()) throw new Error('Wrong AWS profile; no change submitted');
    state.awsLoginVerifiedAt = new Date().toISOString();
    if (state.status === 'verification_submitted') { persist(); throw new Error('Prior submission uncertain; no automatic resubmission'); }
    if (state.status.includes('failed') && !process.argv.includes('--retry')) { persist(); throw new Error('Previous failure recorded; inspect then use --retry for one attempt'); }
    if (!state.alias) {
      const id = 'kiro-migration-' + require('node:crypto').createHash('sha256').update(email.toLowerCase()).digest('hex').slice(0, 24);
      const result = await api('/aliases/batch', { account_ids: [id] });
      state.alias = result.aliases?.find(a => a.account_id === id && a.status === 'active');
      if (!state.alias) throw new Error('No active dedicated alias returned');
      state.status = 'alias_allocated'; persist();
    }
    const status = await changeEmail(page, { email, state, persist, getCode: newCode });
    console.log('MIGRATION_RESULT: ' + status);
    if (status !== 'profile_email_changed') throw new Error('AWS rejected email change; see receipt and browser');
  } catch (error) {
    state.lastFailure = safeError(error, [env.BUILDER_PASS, env.MAILBOX_PASSWORD, token]);
    if (error.awsCode) {
      state.loginErrorCode = error.awsCode;
      state.loginRequestId = error.requestId;
    }
    persist();
    if (page && !page.isClosed()) await page.bringToFront().catch(() => {});
    console.error('MIGRATION_STOPPED: ' + safeError(error, [env.BUILDER_PASS, env.MAILBOX_PASSWORD, token]));
    process.exitCode = 1;
    // Keep the live error page available, as requested. Ctrl+C ends an owned browser.
    if (browser && !env.AWS_CDP_URL && env.KEEP_OPEN_ON_ERROR !== '0') {
      console.log('BROWSER_KEPT_OPEN: inspect the page; close Chrome or press Ctrl+C to exit');
      await new Promise(resolve => browser.once('disconnected', resolve));
    }
  } finally { if (browser?.isConnected()) await browser.close(); }
}
function safeError(error, secrets) {
  let message = String(error.message || error).split('\n')[0];
  for (const secret of secrets) if (secret) message = message.split(secret).join('[redacted]');
  return message;
}
if (require.main === module) main().catch(() => { console.error('MIGRATION_FAILED: check configuration or state file'); process.exitCode = 1; });
module.exports = { apiCode, trustedAWS, outcome, updateLedger, changeEmail, awsLogin, safeError,
  save, mailbox, profileEmail, outlookCodes, oldEmailCode };
