#!/usr/bin/env node
// Work/school Microsoft accounts only. No secrets or page dumps in logs/receipts.
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require('playwright');
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));

function save(file, data) {
  fs.mkdirSync(path.dirname(file), { recursive: true, mode: 0o700 });
  fs.writeFileSync(file + '.tmp', JSON.stringify(data, null, 2), { mode: 0o600 });
  fs.renameSync(file + '.tmp', file);
}

// A submitted request is ambiguous until a fresh login succeeds. Never submit
// another change automatically after a crash or a verification failure.
async function changeAndVerify({ email, receipt, change, verify, alreadyStandard = false }) {
  const state = fs.existsSync(receipt) ? JSON.parse(fs.readFileSync(receipt, 'utf8')) : null;
  if (state && (state.email !== email || !['submitted', 'verified'].includes(state.status)))
    throw new Error('Password receipt does not match this account');
  if (!state && !alreadyStandard) {
    await change(() => save(receipt, { email, status: 'submitted', at: new Date().toISOString() }));
  }
  await verify();
  save(receipt, { email, status: 'verified', at: new Date().toISOString() });
}

async function firstVisible(locator) {
  for (const item of await locator.all()) if (await item.isVisible()) return item;
  return null;
}
async function click(page, locator) {
  const item = await firstVisible(locator);
  if (!item || !await item.isEnabled()) return false;
  await item.click();
  return true;
}
function trusted(url) {
  const u = new URL(url);
  return u.protocol === 'https:' && ['login.microsoftonline.com', 'myaccount.microsoft.com',
    'mysignins.microsoft.com', 'account.activedirectory.windowsazure.com', 'outlook.office.com', 'outlook.office365.com'].includes(u.hostname);
}
async function login(page, email, password, destination) {
  await page.goto(destination, { waitUntil: 'domcontentloaded', timeout: 60000 });
  let emailSent = false, passwordSent = false, notified = false, lastDiagnostic = 0;
  const deadline = Date.now() + 180000;
  while (Date.now() < deadline) {
    if (!trusted(page.url())) throw new Error('Unexpected Microsoft login destination; inspect manually');
    const body = await page.locator('body').innerText();
    if (Date.now() - lastDiagnostic > 30000) {
      lastDiagnostic = Date.now();
      console.log('OUTLOOK_LOGIN_STATE: ' + JSON.stringify({host: new URL(page.url()).hostname, emailSent, passwordSent}));
    }
    if (/password is incorrect|incorrect.*password|account.*locked|couldn.t find an account|account or password is incorrect|帐户或密码不正确|账户或密码不正确|密码不正确|帐户已锁定/i.test(body))
      throw new Error('Microsoft rejected the login; no automatic password retry');
    const host = new URL(page.url()).hostname;
    if (destination.includes('outlook.office.com') && /^outlook\.office(365)?\.com$/.test(host) &&
        body.toLowerCase().includes(email.toLowerCase()) &&
        (await page.getByRole('listbox').count() ||
         (await page.getByRole('main').count() && /Enjoy your empty inbox/i.test(body)))) return;
    if (destination.includes('myaccount.microsoft.com') && host === 'myaccount.microsoft.com' &&
        body.toLowerCase().includes(email.toLowerCase()) &&
        (await firstVisible(page.getByRole('button', { name: /^Change how you sign in$|^更改登录方式$|^Ubah cara Anda masuk/i })) ||
         await firstVisible(page.getByRole('link', { name: /^change password$|^更改密码$/i })))) return;
    if (/Pick an account|选择帐户|选择账户/i.test(body)) {
      await click(page, page.getByText(email, {exact:true}));
    } else if (/Stay signed in|保持登录状态/i.test(body)) {
      await click(page, page.getByRole('button', { name: /^No$|^否$/i }));
    } else if (/Enter password|输入密码/i.test(body) && !passwordSent) {
      const input = await firstVisible(page.locator('input[name="passwd"]:not([aria-hidden="true"])'));
      if (input) {
        await input.fill(password);
        if (await input.inputValue() !== password) throw new Error('Microsoft password input mismatch');
        passwordSent = await click(page, page.locator('#idSIButton9'));
      }
    } else if (!emailSent && !/Enter password|输入密码/i.test(body)) {
      const input = await firstVisible(page.locator('input[name="loginfmt"]:not([aria-hidden="true"])'));
      if (input) {
        await input.fill(email);
        emailSent = await click(page, page.locator('#idSIButton9'));
      }
    }
    if (!notified && /approve sign in|verify your identity|enter code|authenticator|more information required|验证你的身份|验证您的身份|批准登录|输入代码|需要更多信息/i.test(body)) {
      console.log('OUTLOOK_HUMAN_REQUIRED: complete Microsoft verification in Chrome (AWS TOTP is not reused)');
      await page.bringToFront(); notified = true;
    }
    await pause(1000);
  }
  throw new Error('Microsoft login/identity verification timed out');
}

async function submitPasswordChange(page, { email, oldPassword, newPassword, beforeSubmit, verificationTimeoutMs = 600000 }) {
  // Observed school-account UI: My Account -> Change how you sign in ->
  // mysignins.microsoft.com/security-info -> Change Password -> alertdialog.
  const security = page.getByRole('button', { name: /^Change how you sign in$|^更改登录方式$|^Ubah cara Anda masuk/i });
  if (await firstVisible(security)) {
    await security.click();
    const change = page.getByRole('button', { name: /^Change password$|^更改\s*密码$/i });
    try {
      await change.waitFor({ state: 'visible', timeout: 30000 });
    } catch (error) {
      // Security info can require an additional approval after My Account login.
      if (new URL(page.url()).hostname !== 'login.microsoftonline.com' ||
          !/approve sign in request|verify your identity|enter code|批准登录|验证.*身份/i.test(await page.locator('body').innerText())) throw error;
      console.log('OUTLOOK_HUMAN_REQUIRED: approve the Security info sign-in in the linked app');
      await page.bringToFront();
      try {
        await change.waitFor({ state: 'visible', timeout: verificationTimeoutMs });
      } catch (approvalError) {
        if (approvalError.name !== 'TimeoutError') throw approvalError;
        throw new Error('Microsoft Authenticator approval timed out; password change not submitted');
      }
    }
    if (!trusted(page.url())) throw new Error('Unexpected password change destination');
    await change.click();
  } else if (!await click(page, page.getByRole('link', { name: /^change password$|^更改密码$/i }))) {
    throw new Error('Change password entry unavailable');
  }
  const dialog = page.locator('[role=dialog], [role=alertdialog]').filter({ has: page.locator('input[type=password]') });
  await dialog.locator('input[type=password]').first().waitFor({ state: 'visible', timeout: 30000 });
  if (!trusted(page.url()) || !(await dialog.innerText()).toLowerCase().includes(email.toLowerCase()))
    throw new Error('Password form destination or identity mismatch');
  const old = dialog.getByLabel(/^current password$|^old password$|^当前密码$/i);
  const next = dialog.getByLabel(/^new password$|^新密码$/i);
  const confirm = dialog.getByLabel(/^confirm new password$|^确认新密码$/i);
  for (const [field, value] of [[old, oldPassword], [next, newPassword], [confirm, newPassword]]) {
    await field.fill('');
    await field.pressSequentially(value, { delay: 40 });
    await field.press('Tab');
    await pause(150);
    if (await field.inputValue() !== value) throw new Error('Password input mismatch');
  }
  if (await old.inputValue() !== oldPassword || await next.inputValue() !== newPassword ||
      await confirm.inputValue() !== newPassword) throw new Error('Password input mismatch');
  const submit = dialog.getByRole('button', { name: /^submit$|^提交$/i });
  if (!await submit.isEnabled()) throw new Error('Microsoft password form rejected the input');
  beforeSubmit();
  await submit.click();
  // A dismissed dialog is only a submission acknowledgement. The caller still
  // requires a new-context login; a policy error leaves the dialog open and fails.
  await dialog.locator('input[type=password]').first().waitFor({ state: 'hidden', timeout: 30000 });
}

async function main() {
  process.umask(0o077);
  const email = process.env.MAILBOX_EMAIL;
  const oldPassword = process.env.MAILBOX_PASSWORD;
  const newPassword = process.env.MAILBOX_NEW_PASSWORD;
  const dir = process.env.ARTIFACT_DIR;
  if (!email || !oldPassword || !newPassword || !dir) throw new Error('Missing mailbox email, old/new password or ARTIFACT_DIR');
  const browser = await chromium.launch({ channel: 'chrome', headless: false,
    proxy: { server: process.env.OAUTH_PROXY || 'http://127.0.0.1:7890' } });
  try {
    await changeAndVerify({ email, receipt: path.join(dir, 'outlook-password.json'),
      alreadyStandard: oldPassword === newPassword,
      change: async beforeSubmit => {
        const context = await browser.newContext({ locale: 'en-US' });
        try {
          const page = await context.newPage();
          console.log('OUTLOOK_STAGE: login-myaccount');
          await login(page, email, oldPassword, 'https://myaccount.microsoft.com/');
          console.log('OUTLOOK_STAGE: change-form');
          await submitPasswordChange(page, { email, oldPassword, newPassword, beforeSubmit });
        } catch (error) {
          const page = context.pages()[0];
          if (page) {
            console.error('OUTLOOK_PAGE_HOST: ' + new URL(page.url()).hostname);
          }
          throw error;
        } finally { await context.close(); }
      },
      verify: async () => {
        const context = await browser.newContext({ locale: 'en-US' });
        try {
          console.log('OUTLOOK_STAGE: verify-new-password');
          await login(await context.newPage(), email, newPassword, 'https://outlook.office.com/mail/');
        } finally { await context.close(); }
      },
    });
    console.log('OUTLOOK_PASSWORD_VERIFIED: ' + email);
  } finally { await browser.close(); }
}
if (require.main === module) main().then(() => process.exit(0)).catch(error => {
  // Playwright errors may include form values. Keep diagnostics credential-free.
  const known = /^(Microsoft rejected the login|Unexpected Microsoft login destination|Microsoft login\/identity verification timed out|Change password entry unavailable|Password form destination or identity mismatch|Unexpected password change destination|Microsoft password form rejected the input|Password receipt does not match this account)/;
  let detail = known.test(error.message) ? error.message : String(error.message || error.name).split('\n')[0];
  for (const secret of [process.env.MAILBOX_PASSWORD, process.env.MAILBOX_NEW_PASSWORD])
    if (secret) detail = detail.split(secret).join('[redacted]');
  console.error('OUTLOOK_FAILURE_KIND: ' + detail);
  console.error('OUTLOOK_PASSWORD_STOPPED: change/login not verified. Inspect Microsoft manually; if a receipt exists, rerun only verifies the supplied new password.');
  process.exitCode = 1;
});
module.exports = { changeAndVerify, trusted, login, submitPasswordChange };
