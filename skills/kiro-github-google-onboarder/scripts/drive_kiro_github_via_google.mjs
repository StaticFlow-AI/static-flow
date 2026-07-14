#!/usr/bin/env node

import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  configuredDelayBounds,
  randomDelayMs,
} from "../../kiro-social-onboarder/scripts/github_step_delay.mjs";

const port = process.env.KIRO_DEVTOOLS_PORT;
const googleEmail = process.env.KIRO_GOOGLE_EMAIL || "";
const googlePassword = process.env.KIRO_GOOGLE_PASSWORD || "";
const googleTotpSecret = process.env.KIRO_GOOGLE_TOTP_SECRET || "";
const githubTotpSecret = process.env.KIRO_GITHUB_TOTP_SECRET || "";
const timeoutSeconds = Number(process.env.KIRO_MANUAL_TIMEOUT_SECONDS || "900");
const isMain =
  process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1]);

const sleep = (ms) => new Promise((resolveSleep) => setTimeout(resolveSleep, ms));
const stepDelayBounds = configuredDelayBounds();

async function randomStepDelay(label) {
  const delayMs = randomDelayMs(stepDelayBounds);
  if (delayMs <= 0) return;
  console.log(`Browser helper: waiting ${delayMs}ms ${label}`);
  await sleep(delayMs);
}

function jsString(value) {
  return JSON.stringify(value);
}

function extractTotpCodeFrom2faFunValues(values) {
  for (const value of values) {
    const match = String(value || "").match(/^\s*(\d{6})\s*$/);
    if (match) return match[1];
  }
  return "";
}

function totpCodeForRemaining(values, remaining) {
  const code = extractTotpCodeFrom2faFunValues(values);
  return code && Number(remaining) >= 15 ? code : "";
}

function isGoogleTotpPrompt(host, path, inputNames, text) {
  return (
    host === "accounts.google.com" &&
    (path.includes("/challenge/totp") ||
      inputNames.includes("totpPin"))
  );
}

function isGoogleChallengeSelection(host, path) {
  return host === "accounts.google.com" && path.includes("/challenge/selection");
}

function isGithubTotpPrompt(host, path, inputNames, text) {
  const lower = String(text || "").toLowerCase();
  return (
    host === "github.com" &&
    (path.includes("two-factor") ||
      inputNames.some((name) => ["app_otp", "otp"].includes(name)) ||
      lower.includes("authentication code"))
  );
}

function isManualGoogleChallenge(host, path, text) {
  if (host !== "accounts.google.com") return false;
  const lower = String(text || "").toLowerCase();
  return (
    path.includes("/challenge/") ||
    lower.includes("verify it’s you") ||
    lower.includes("verify it's you") ||
    lower.includes("check your phone") ||
    lower.includes("passkey") ||
    lower.includes("security key") ||
    lower.includes("captcha") ||
    lower.includes("account recovery") ||
    lower.includes("couldn’t verify") ||
    lower.includes("couldn't verify")
  );
}

async function browserTargets() {
  return await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
}

function selectAuthTarget(targets) {
  return targets.find(
    (target) =>
      target.type === "page" &&
      !target.url.includes("2fa.fun") &&
      (target.url.includes("accounts.google.com") ||
        target.url.includes("github.com") ||
        target.url.includes("kiro"))
  ) || targets.find((target) => target.type === "page" && !target.url.includes("2fa.fun"));
}

let ws;
let currentTargetId = "";
let nextId = 0;
let pending = new Map();

async function connectTarget(target) {
  if (!target?.webSocketDebuggerUrl) {
    throw new Error("Chrome DevTools page target not found");
  }
  if (ws) ws.close();
  ws = new WebSocket(target.webSocketDebuggerUrl);
  currentTargetId = target.id;
  nextId = 0;
  pending = new Map();
  ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    if (message.id && pending.has(message.id)) {
      pending.get(message.id)(message);
      pending.delete(message.id);
    }
  };
  await new Promise((resolveOpen, reject) => {
    ws.onopen = resolveOpen;
    ws.onerror = reject;
  });
  await send("Runtime.enable");
  await send("Page.enable");
  await send("Page.bringToFront");
}

async function syncAuthTarget() {
  const deadline = Date.now() + 25_000;
  while (Date.now() < deadline) {
    const target = selectAuthTarget(await browserTargets());
    if (target?.webSocketDebuggerUrl) {
      if (target.id !== currentTargetId || !ws || ws.readyState !== WebSocket.OPEN) {
        await connectTarget(target);
      }
      return;
    }
    await sleep(250);
  }
  throw new Error("Chrome DevTools page target not found");
}

function send(method, params = {}) {
  return new Promise((resolveSend, reject) => {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      reject(new Error("Chrome DevTools websocket is not connected"));
      return;
    }
    const id = ++nextId;
    pending.set(id, resolveSend);
    ws.send(JSON.stringify({ id, method, params }));
  });
}

async function evalJs(expression) {
  const response = await send("Runtime.evaluate", {
    expression,
    returnByValue: true,
    awaitPromise: true,
  });
  if (response.exceptionDetails) {
    throw new Error(JSON.stringify(response.exceptionDetails));
  }
  return response.result?.result?.value;
}

async function state() {
  return await evalJs(`(() => ({
    title: document.title,
    host: location.host,
    path: location.pathname,
    text: document.body ? document.body.innerText.slice(0, 3200) : '',
    inputs: [...document.querySelectorAll('input')].map((e) => ({
      id: e.id || '',
      name: e.name || '',
      type: e.type || '',
      autocomplete: e.autocomplete || '',
      visible: !!(e.offsetWidth || e.offsetHeight || e.getClientRects().length),
    })),
    buttons: [...document.querySelectorAll('button,a,[role="button"],input[type="submit"]')]
      .map((e) => (e.innerText || e.value || e.getAttribute('aria-label') || '').trim())
      .filter(Boolean)
      .slice(0, 100),
  }))()`);
}

async function setInput(selector, value) {
  return await evalJs(`(() => {
    const element = document.querySelector(${jsString(selector)});
    if (!element) return false;
    element.scrollIntoView({ block: 'center' });
    element.focus();
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
    if (setter) setter.call(element, ${jsString(value)}); else element.value = ${jsString(value)};
    element.dispatchEvent(new Event('input', { bubbles: true }));
    element.dispatchEvent(new Event('change', { bubbles: true }));
    return true;
  })()`);
}

async function clickStructural(selectors) {
  return await evalJs(`(() => {
    for (const selector of ${jsString(selectors)}) {
      const element = [...document.querySelectorAll(selector)].find((candidate) =>
        !!(candidate.offsetWidth || candidate.offsetHeight || candidate.getClientRects().length) &&
        !candidate.disabled &&
        candidate.getAttribute('aria-disabled') !== 'true'
      );
      if (!element) continue;
      element.scrollIntoView({ block: 'center' });
      element.click();
      return true;
    }
    return false;
  })()`);
}

async function clickText(label, contains = false) {
  return await evalJs(`(() => {
    const target = ${jsString(label)}.toLowerCase();
    const elements = [...document.querySelectorAll('button,a,[role="button"],input[type="submit"]')];
    const element = elements.find((candidate) => {
      const value = (candidate.innerText || candidate.value || candidate.getAttribute('aria-label') || '').trim().toLowerCase();
      const matched = ${contains} ? value.includes(target) : value === target;
      return matched && !candidate.disabled && candidate.getAttribute('aria-disabled') !== 'true';
    });
    if (!element) return false;
    element.scrollIntoView({ block: 'center' });
    element.click();
    return true;
  })()`);
}

async function clickGithubGoogleLogin() {
  return await evalJs(`(() => {
    const selectors = [
      '[data-provider="google"]',
      'button[data-oauth-provider="google"]',
      'a[href*="google"][href*="login"]',
      'a[href*="google"][href*="auth"]',
    ];
    for (const selector of selectors) {
      const element = document.querySelector(selector);
      if (element && !!(element.offsetWidth || element.offsetHeight || element.getClientRects().length)) {
        element.click();
        return true;
      }
    }
    const element = [...document.querySelectorAll('button,a,[role="button"]')].find((candidate) => {
      const label = (candidate.innerText || candidate.getAttribute('aria-label') || '').trim().toLowerCase();
      return label.includes('google') && (label.includes('continue') || label.includes('sign in'));
    });
    if (!element) return false;
    element.click();
    return true;
  })()`);
}

async function evaluateInTarget(target, expression) {
  const targetWs = new WebSocket(target.webSocketDebuggerUrl);
  let targetId = 0;
  const targetPending = new Map();
  targetWs.onmessage = (event) => {
    const message = JSON.parse(event.data);
    if (message.id && targetPending.has(message.id)) {
      targetPending.get(message.id)(message);
      targetPending.delete(message.id);
    }
  };
  await new Promise((resolveOpen, reject) => {
    targetWs.onopen = resolveOpen;
    targetWs.onerror = reject;
  });
  const targetSend = (method, params = {}) =>
    new Promise((resolveSend) => {
      const id = ++targetId;
      targetPending.set(id, resolveSend);
      targetWs.send(JSON.stringify({ id, method, params }));
    });
  await targetSend("Page.bringToFront");
  const response = await targetSend("Runtime.evaluate", {
    expression,
    returnByValue: true,
    awaitPromise: true,
  });
  targetWs.close();
  if (response.exceptionDetails) {
    throw new Error(JSON.stringify(response.exceptionDetails));
  }
  return response.result?.result?.value;
}

async function openOrFind2faFun() {
  let target = (await browserTargets()).find(
    (item) => item.type === "page" && item.url.includes("2fa.fun")
  );
  if (!target) {
    await fetch(
      `http://127.0.0.1:${port}/json/new?${encodeURIComponent("https://2fa.fun/")}`,
      { method: "PUT" }
    );
  }
  const deadline = Date.now() + 20_000;
  while (Date.now() < deadline) {
    target = (await browserTargets()).find(
      (item) => item.type === "page" && item.url.includes("2fa.fun")
    );
    if (target?.webSocketDebuggerUrl) return target;
    await sleep(500);
  }
  throw new Error("2fa.fun page did not open");
}

async function codeFrom2faFun(secret) {
  const target = await openOrFind2faFun();
  const readyDeadline = Date.now() + 20_000;
  while (Date.now() < readyDeadline) {
    const ready = await evaluateInTarget(
      target,
      `!!document.querySelector('#SECRET2FA,textarea[name="SECRET2FA"],textarea')`
    );
    if (ready) break;
    await sleep(500);
  }
  const submitted = await evaluateInTarget(
    target,
    `(() => {
      const textarea = document.querySelector('#SECRET2FA,textarea[name="SECRET2FA"],textarea');
      if (!textarea) return false;
      textarea.focus();
      const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value')?.set;
      if (setter) setter.call(textarea, ${jsString(secret)}); else textarea.value = ${jsString(secret)};
      textarea.dispatchEvent(new Event('input', { bubbles: true }));
      textarea.dispatchEvent(new Event('change', { bubbles: true }));
      const button = [...document.querySelectorAll('button,input[type="submit"],[role="button"]')]
        .find((candidate) => /获取验证码|验证码|get codes?|code/i.test(
          (candidate.innerText || candidate.value || candidate.getAttribute('aria-label') || '').trim()
        ));
      if (!button) return false;
      button.click();
      return true;
    })()`
  );
  if (!submitted) throw new Error("2fa.fun form submission failed");

  const deadline = Date.now() + 65_000;
  while (Date.now() < deadline) {
    await sleep(500);
    const otpState = await evaluateInTarget(
      target,
      `(() => {
        const values = [...document.querySelectorAll('input.faotp')].map((e) => e.value || '');
        const timerText = document.querySelector('.time2fa')?.textContent || '';
        const remaining = Number((timerText.match(/\\d+/) || ['0'])[0]);
        return { values, remaining };
      })()`
    );
    const code = totpCodeForRemaining(otpState?.values || [], otpState?.remaining || 0);
    if (code) return code;
    const remaining = Number(otpState?.remaining || 0);
    if (remaining > 0 && remaining < 15) {
      await sleep((remaining + 1) * 1000);
      await evaluateInTarget(
        target,
        `(() => {
          const textarea = document.querySelector('#SECRET2FA,textarea[name="SECRET2FA"],textarea');
          if (!textarea) return false;
          const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value')?.set;
          if (setter) setter.call(textarea, ${jsString(secret)}); else textarea.value = ${jsString(secret)};
          textarea.dispatchEvent(new Event('input', { bubbles: true }));
          textarea.dispatchEvent(new Event('change', { bubbles: true }));
          const button = [...document.querySelectorAll('button,input[type="submit"],[role="button"]')]
            .find((candidate) => /获取验证码|验证码|get codes?|code/i.test(
              (candidate.innerText || candidate.value || candidate.getAttribute('aria-label') || '').trim()
            ));
          if (!button) return false;
          button.click();
          return true;
        })()`
      );
    }
  }
  throw new Error("2fa.fun did not provide a code with a safe validity window");
}

async function main() {
  process.exitCode = 1;
  if (!port || !googleEmail || !googlePassword || !googleTotpSecret || !githubTotpSecret) {
    throw new Error("DevTools port, Google credentials, and both TOTP secrets are required");
  }

  await syncAuthTarget();
  const deadline = Date.now() + timeoutSeconds * 1000;
  let lastAction = "started";
  let lastManualNoticeAt = 0;
  let clickedGithubGoogle = false;
  let selectedGoogleAccount = false;
  let submittedGoogleEmail = false;
  let submittedGooglePassword = false;
  let selectedGoogleTotpMethod = false;
  let submittedGoogleTotp = false;
  let submittedGithubTotp = false;
  let approvedGoogleConsent = false;
  let approvedGithubConsent = false;

  while (Date.now() < deadline) {
    await syncAuthTarget();
    const current = await state();
    const host = current.host || "";
    const path = current.path || "";
    const text = current.text || "";
    const lower = text.toLowerCase();
    const buttons = current.buttons || [];
    const inputNames = (current.inputs || []).filter((input) => input.visible).map((input) => input.name);

    if (
      lower.includes("device authorized") ||
      lower.includes("authorization complete") ||
      lower.includes("you may close this window")
    ) {
      console.log("Browser helper: Kiro device authorized");
      ws.close();
      process.exitCode = 0;
      return;
    }

    if (lower.includes("something went wrong") && buttons.includes("Restart")) {
      await randomStepDelay("before Kiro Restart");
      await clickText("Restart");
      lastAction = "restarted Kiro authorization";
      await sleep(2500);
      continue;
    }

    if (lower.includes("authorization requested")) {
      await randomStepDelay("before Kiro approval");
      await clickText("Accept");
      await sleep(500);
      await clickText("Approve");
      lastAction = "approved Kiro device";
      await sleep(2000);
      continue;
    }

    if (!host.includes("github.com") && host !== "accounts.google.com" && buttons.includes("Continue")) {
      await randomStepDelay("before Kiro Continue");
      await clickText("Continue");
      lastAction = "continued Kiro authorization";
      await sleep(2500);
      continue;
    }

    const githubLoginPage =
      host === "github.com" &&
      (path.includes("/login") || lower.includes("sign in to github"));
    if (githubLoginPage && !clickedGithubGoogle) {
      await randomStepDelay("before GitHub Continue with Google");
      const clicked = await clickGithubGoogleLogin();
      if (clicked) {
        clickedGithubGoogle = true;
        lastAction = "selected GitHub Continue with Google";
        console.log("Browser helper: selected GitHub sign-in through Google");
        await sleep(3500);
        continue;
      }
    }

    if (host === "accounts.google.com" && lower.includes("choose an account") && !selectedGoogleAccount) {
      await randomStepDelay("before selecting Google account");
      const selected = await clickText(googleEmail);
      selectedGoogleAccount = true;
      lastAction = `selected Google account=${selected}`;
      await sleep(3000);
      continue;
    }

    const hasGoogleEmail = (current.inputs || []).some(
      (input) => input.visible && (input.id === "identifierId" || input.type === "email")
    );
    if (host === "accounts.google.com" && hasGoogleEmail && !submittedGoogleEmail) {
      await randomStepDelay("before Google email");
      const filled = await setInput('#identifierId,input[type="email"],input[name="identifier"]', googleEmail);
      const submitted = await clickStructural([
        "#identifierNext button",
        "#identifierNext",
        'button[jsname="LgbsSe"].nCP5yc',
        'button[type="submit"]',
      ]);
      submittedGoogleEmail = true;
      lastAction = `submitted Google email filled=${filled} submitted=${submitted}`;
      console.log("Browser helper: submitted Google email");
      await sleep(3500);
      continue;
    }

    const googlePasswordInput = (current.inputs || []).some(
      (input) => input.visible && input.type === "password"
    );
    if (host === "accounts.google.com" && googlePasswordInput && !submittedGooglePassword) {
      await randomStepDelay("before Google password");
      const filled = await setInput(
        'input[name="Passwd"],input[autocomplete="current-password"],input[type="password"]',
        googlePassword
      );
      const submitted = await clickStructural([
        "#passwordNext button",
        "#passwordNext",
        'button[jsname="LgbsSe"].nCP5yc',
        'button[type="submit"]',
      ]);
      submittedGooglePassword = true;
      lastAction = `submitted Google password filled=${filled} submitted=${submitted}`;
      console.log("Browser helper: submitted Google password");
      await sleep(3500);
      continue;
    }

    if (
      isGoogleTotpPrompt(host, path, inputNames, text) &&
      !submittedGoogleTotp
    ) {
      console.log("Browser helper: obtaining Google authenticator code");
      const code = await codeFrom2faFun(googleTotpSecret);
      await randomStepDelay("before Google authenticator code");
      const filled = await setInput(
        'input#totpPin,input[name="totpPin"],input[autocomplete="one-time-code"],input[type="tel"]',
        code
      );
      const submitted = await clickStructural([
        "#totpNext button",
        "#totpNext",
        'button[jsname="LgbsSe"].nCP5yc',
        'button[type="submit"]',
      ]);
      submittedGoogleTotp = true;
      lastAction = `submitted Google TOTP filled=${filled} submitted=${submitted}`;
      console.log("Browser helper: submitted Google authenticator code");
      await sleep(4000);
      continue;
    }

    if (
      isGoogleChallengeSelection(host, path) &&
      !selectedGoogleTotpMethod
    ) {
      await randomStepDelay("before selecting Google Authenticator");
      const selected = await clickStructural([
        '[data-action="selectchallenge"][data-challengetype="6"]:not([aria-disabled="true"])',
      ]);
      selectedGoogleTotpMethod = true;
      lastAction = `selected Google authenticator method=${selected}`;
      console.log("Browser helper: selected Google authenticator method");
      await sleep(3500);
      continue;
    }

    const googleConsent =
      host === "accounts.google.com" &&
      (path.includes("/oauth/") || lower.includes("sign in to github"));
    if (googleConsent && !approvedGoogleConsent && !isManualGoogleChallenge(host, path, text)) {
      await randomStepDelay("before Google consent");
      const approved = await clickStructural([
        'button[jsname="LgbsSe"].nCP5yc',
        'button[type="submit"]',
      ]);
      approvedGoogleConsent = true;
      lastAction = `approved Google consent=${approved}`;
      await sleep(3500);
      continue;
    }

    if (isManualGoogleChallenge(host, path, text) && !isGoogleTotpPrompt(host, path, inputNames, text)) {
      if (Date.now() - lastManualNoticeAt > 10_000) {
        console.log("Browser helper: manual Google verification required; complete it in the browser");
        lastManualNoticeAt = Date.now();
      }
      lastAction = "waiting for manual Google verification";
      await sleep(2000);
      continue;
    }

    if (isGithubTotpPrompt(host, path, inputNames, text) && !submittedGithubTotp) {
      console.log("Browser helper: obtaining GitHub authenticator code");
      const code = await codeFrom2faFun(githubTotpSecret);
      await randomStepDelay("before GitHub authenticator code");
      const filled = await setInput(
        'input[name="app_otp"],input#app_totp,input[name="otp"],input[autocomplete="one-time-code"]',
        code
      );
      const submitted = await clickStructural([
        'button[type="submit"]',
        'input[type="submit"]',
      ]);
      submittedGithubTotp = true;
      lastAction = `submitted GitHub TOTP filled=${filled} submitted=${submitted}`;
      console.log("Browser helper: submitted GitHub authenticator code");
      await sleep(4000);
      continue;
    }

    const githubConsent =
      host === "github.com" &&
      (path.includes("/login/oauth/authorize") || lower.includes("authorize"));
    if (githubConsent && !approvedGithubConsent) {
      await randomStepDelay("before GitHub OAuth approval");
      const approved = await clickStructural([
        'button[name="authorize"]',
        'input[name="authorize"]',
        'button[type="submit"].btn-primary',
      ]);
      approvedGithubConsent = true;
      lastAction = `approved GitHub OAuth=${approved}`;
      console.log("Browser helper: approved GitHub OAuth");
      await sleep(3500);
      continue;
    }

    if (host === "github.com") {
      if (Date.now() - lastManualNoticeAt > 10_000) {
        console.log("Browser helper: manual GitHub verification or consent required");
        lastManualNoticeAt = Date.now();
      }
      lastAction = "waiting for manual GitHub step";
      await sleep(2000);
      continue;
    }

    await sleep(1000);
  }

  const finalState = await state();
  ws.close();
  throw new Error(
    `Browser helper timed out; lastAction=${lastAction}; title=${finalState.title}; host=${finalState.host}; path=${finalState.path}`
  );
}

export {
  extractTotpCodeFrom2faFunValues,
  isGithubTotpPrompt,
  isGoogleChallengeSelection,
  isGoogleTotpPrompt,
  isManualGoogleChallenge,
  selectAuthTarget,
  totpCodeForRemaining,
};

if (isMain) {
  main().catch((error) => {
    console.error(error?.stack || error?.message || String(error));
    process.exit(1);
  });
}
