// Attach to an operator-owned, visible Chrome; receive credentials only over private stdin.
const fs = require('node:fs');
const path = require('node:path');
const readline = require('node:readline');
const { createRequire } = require('node:module');
const batchRequire = createRequire(path.join(__dirname, 'batch/package.json'));
const { chromium } = batchRequire('playwright');
const { trusted, login, submitPasswordChange, changeAndVerify } = require('./batch/scripts/outlook_change_password.cjs');
const { loginProxyForPosition } = require('./login-network-policy.cjs');
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
const paced = Symbol('outlookActionPacing');
let observeAction = () => {};

async function loginMicrosoft(page, email, password, destination) {
  try { return await login(page, email, password, destination); }
  catch (error) {
    // Microsoft now redirects Outlook to this documented endpoint. Preserve the
    // supplied snapshot and accept only the authenticated mailbox destination.
    // https://learn.microsoft.com/en-us/microsoft-365/enterprise/urls-and-ip-address-ranges
    const url = new URL(page.url());
    if (error.message !== 'Unexpected Microsoft login destination; inspect manually' ||
        destination !== 'https://outlook.office.com/mail/' ||
        url.origin !== 'https://outlook.cloud.microsoft' || !url.pathname.startsWith('/mail/')) throw error;
    await page.getByRole('listbox').first().waitFor({ state: 'visible', timeout: 30000 });
    await page.getByText(email, { exact: true }).first().waitFor({ state: 'visible', timeout: 30000 });
  }
}

function pacePage(page) {
  const targets = [
    [Object.getPrototypeOf(page), ['goto', 'reload', 'click', 'fill', 'press', 'type', 'bringToFront']],
    [Object.getPrototypeOf(page.locator('body')), ['click', 'fill', 'press', 'pressSequentially', 'check', 'uncheck', 'selectOption']],
    [Object.getPrototypeOf(page.keyboard), ['press', 'type', 'insertText']],
  ];
  for (const [prototype, methods] of targets) {
    if (Object.hasOwn(prototype, paced)) continue;
    for (const name of methods) {
      const original = prototype[name];
      if (typeof original !== 'function') continue;
      prototype[name] = async function (...args) {
        try {
          const targetId = name === 'click' && state.stage === 'login_current_password' && typeof this.page === 'function' &&
            typeof this.getAttribute === 'function' ? await this.getAttribute('id', { timeout: 3000 }) : null;
          const result = await original.apply(this, args);
          await observeAction(name, args, this, targetId);
          return result;
        }
        finally { await delay(2000); }
      };
    }
    Object.defineProperty(prototype, paced, { value: true });
  }
  return page;
}

let state = { stage: 'ready' }, browser, verificationContext, job, secrets = [];
const emit = value => process.stdout.write(JSON.stringify(value) + '\n');
function safeError(error) {
  let text = String(error.message || error).split('\n')[0];
  for (const value of secrets) if (value) text = text.split(value).join('[redacted]');
  return text.replace(/https?:\/\/\S+/g, value => {
    try { const u = new URL(value); return u.origin + u.pathname + (u.search ? '?[redacted]' : ''); }
    catch { return '[url]'; }
  }).slice(0, 300);
}
const originalLog = console.log;
console.log = (...args) => {
  const message = args.map(String).join(' ');
  if (message.startsWith('OUTLOOK_HUMAN_REQUIRED:')) {
    state.humanRequired = true;
    emit({ account: state.account, mode: 'outlook', stage: 'human_verification_required' });
  } else if (message.startsWith('OUTLOOK_LOGIN_STATE:')) {
    const data = JSON.parse(message.slice('OUTLOOK_LOGIN_STATE:'.length));
    emit({ account: state.account, mode: 'outlook', stage: state.stage, ...data });
  }
};

async function start(input) {
  if (job && state.stage !== 'verified' && state.stage !== 'failed') throw new Error('An Outlook operation is already pending');
  const c = JSON.parse(fs.readFileSync(input.checkpointFile, 'utf8'));
  if (c.account !== input.email || c.loginProxy !== loginProxyForPosition(c.batchIndex)) throw new Error('Account or network-policy mismatch');
  const processArgs = fs.readFileSync('/proc/' + c.browserPid + '/cmdline', 'utf8');
  if (!processArgs.includes('--user-data-dir=' + c.browserProfile)) throw new Error('Owned browser identity mismatch');
  if (c.loginProxy ? !processArgs.includes('--proxy-server=' + c.loginProxy) : !processArgs.includes('--no-proxy-server'))
    throw new Error('Owned browser proxy mismatch');
  for (const file of [input.receipt, input.proof]) {
    if (!path.isAbsolute(file) || file.startsWith(path.resolve(__dirname, '../../..') + path.sep)) throw new Error('Use absolute private evidence paths outside the skill');
  }
  if (!input.password || !input.newPassword) throw new Error('Mailbox credentials are missing');
  secrets = [input.password, input.newPassword];
  state = { account: input.email, stage: 'connecting', humanRequired: false, receipt: input.receipt };
  let verificationPasswordEntered = false;
  let staySignedInClicks = 0, staySignedInStarted = 0;
  observeAction = async (name, args, target, targetId) => {
    if (state.stage === 'verify_new_password_in_fresh_context' &&
        ['fill', 'pressSequentially', 'insertText', 'type'].includes(name) && args[0] === input.newPassword)
      verificationPasswordEntered = true;
    if (state.stage !== 'login_current_password' || name !== 'click' || targetId !== 'idBtn_Back') return;
    const page = target.page();
    if (new URL(page.url()).origin !== 'https://login.microsoftonline.com' ||
        !/Stay signed in/.test(await page.locator('body').innerText())) return;
    staySignedInStarted ||= Date.now();
    if (++staySignedInClicks < 3) return;
    await delay(Math.max(0, 30000 - (Date.now() - staySignedInStarted)));
    const body = await page.locator('body').innerText();
    if (new URL(page.url()).origin === 'https://login.microsoftonline.com' &&
        body.includes(input.email) && /Stay signed in/.test(body))
      throw new Error('Microsoft stay-signed-in page stalled after three clicks');
  };
  job = (async () => {
    try {
      browser = await chromium.connectOverCDP('http://127.0.0.1:' + c.debugPort);
      const context = browser.contexts()[0];
      const pages = context.pages();
      const microsoftPage = pages.find(page => trusted(page.url()) || new URL(page.url()).origin === 'https://outlook.cloud.microsoft');
      const page = pacePage(microsoftPage || pages[0] || await context.newPage());
      context.setDefaultTimeout(30000);
      await changeAndVerify({
        email: input.email, receipt: input.receipt, alreadyStandard: input.password === input.newPassword,
        change: async beforeSubmit => {
          state.stage = 'login_current_password';
          emit({ account: input.email, mode: 'outlook', stage: state.stage });
          await loginMicrosoft(page, input.email, input.password, 'https://myaccount.microsoft.com/');
          state.stage = 'change_password';
          emit({ account: input.email, mode: 'outlook', stage: state.stage });
          await submitPasswordChange(page, { email: input.email, oldPassword: input.password, newPassword: input.newPassword, beforeSubmit, verificationTimeoutMs: 120000 });
        },
        verify: async () => {
          state.stage = 'verify_new_password_in_fresh_context';
          emit({ account: input.email, mode: 'outlook', stage: state.stage });
          verificationContext = await browser.newContext({ locale: 'en-US', ...(c.loginProxy ? { proxy: { server: c.loginProxy } } : {}) });
          const fresh = pacePage(await verificationContext.newPage());
          await loginMicrosoft(fresh, input.email, input.newPassword, 'https://outlook.office.com/mail/');
          if (!verificationPasswordEntered) throw new Error('Fresh sign-in did not exercise the new password; inspect before completion');
          const evidence = {
            account: input.email, newPasswordSigninVerified: true, freshContext: true,
            passwordEnteredInFreshContext: true,
            browserPid: c.browserPid, browserDebugPort: c.debugPort,
            loginProxy: c.loginProxy, actionSpacingMs: 2000, verifiedAt: new Date().toISOString(),
          };
          fs.writeFileSync(input.proof, JSON.stringify(evidence, null, 2), { mode: 0o600 });
          await verificationContext.close(); verificationContext = null;
        },
      });
      state.stage = 'verified'; state.humanRequired = false;
      emit({ account: input.email, mode: 'outlook', stage: 'verified', freshSigninVerified: true });
      input.password = null; input.newPassword = null; secrets = [];
      observeAction = () => {};
    } catch (error) {
      state.error = safeError(error); state.stage = 'failed';
      state.receiptStatus = fs.existsSync(input.receipt) ? JSON.parse(fs.readFileSync(input.receipt, 'utf8')).status : null;
      emit({ account: input.email, mode: 'outlook', stage: 'failed', errorKind: state.error, receiptStatus: state.receiptStatus, humanRequired: state.humanRequired });
    }
  })();
  return { started: true, account: input.email };
}

async function main() {
  process.umask(0o077);
  emit({ ready: true, mode: 'outlook_cdp' });
  const rl = readline.createInterface({ input: process.stdin, terminal: false });
  for await (const line of rl) {
    try {
      const message = JSON.parse(line);
      if (message.op === 'start') emit({ op: message.op, result: await start(message) });
      else if (message.op === 'status') emit({ op: message.op, result: state });
      else if (message.op === 'stop') {
        if (!['verified', 'failed', 'ready'].includes(state.stage)) throw new Error('Inspect the pending operation before stopping');
        secrets = [];
        emit({ op: 'stop', result: { stopped: true, browserRetained: true } });
        process.exit(0);
      } else throw new Error('Unsupported Outlook operation');
    } catch (error) { emit({ error: safeError(error) }); }
  }
  console.log = originalLog;
}

if (require.main === module) main().catch(error => { emit({ error: safeError(error) }); process.exitCode = 1; });
module.exports = { pacePage };
