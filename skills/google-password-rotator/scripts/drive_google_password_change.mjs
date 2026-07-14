#!/usr/bin/env node

import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const port = process.env.GOOGLE_DEVTOOLS_PORT;
const googleEmail = process.env.GOOGLE_EMAIL || "";
const currentPassword = process.env.GOOGLE_CURRENT_PASSWORD || "";
const newPassword = process.env.GOOGLE_NEW_PASSWORD || "";
const settingsUrl =
  process.env.GOOGLE_SETTINGS_URL ||
  "https://myaccount.google.com/signinoptions/password";
const timeoutSeconds = Number(process.env.GOOGLE_MANUAL_TIMEOUT_SECONDS || "900");
const auto2faFun = process.env.GOOGLE_AUTO_2FA_FUN === "1";
const totpSecret = process.env.GOOGLE_TOTP_SECRET || "";
const verificationOnly = process.env.GOOGLE_VERIFICATION_ONLY === "1";
const setEnglishLanguage =
  process.env.GOOGLE_SET_ENGLISH_LANGUAGE !== "0" && !verificationOnly;
const languageUrl = "https://myaccount.google.com/language?hl=en";
const isMain =
  process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1]);

const sleep = (ms) => new Promise((resolveSleep) => setTimeout(resolveSleep, ms));

async function connectPage() {
  const deadline = Date.now() + 25_000;
  while (Date.now() < deadline) {
    try {
      const pages = await (
        await fetch(`http://127.0.0.1:${port}/json/list`)
      ).json();
      const page =
        pages.find(
          (item) =>
            item.type === "page" &&
            (item.url.includes("accounts.google.com") ||
              item.url.includes("myaccount.google.com"))
        ) || pages.find((item) => item.type === "page");
      if (page?.webSocketDebuggerUrl) {
        return page;
      }
    } catch {
      // Chrome may still be starting.
    }
    await sleep(250);
  }
  throw new Error("Chrome DevTools page target not found");
}

let ws;
let nextId = 0;
const pending = new Map();

async function openWebsocket() {
  const page = await connectPage();
  ws = new WebSocket(page.webSocketDebuggerUrl);
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
}

function send(method, params = {}) {
  return new Promise((resolveSend) => {
    if (!ws) {
      throw new Error("Chrome DevTools websocket is not connected");
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

function jsString(value) {
  return JSON.stringify(value);
}

async function navigate(url) {
  await send("Page.navigate", { url });
}

async function state() {
  return await evalJs(`(() => ({
    title: document.title,
    url: location.href,
    text: document.body ? document.body.innerText.slice(0, 4000) : "",
    emailInput: [...document.querySelectorAll('#identifierId,input[type="email"],input[name="identifier"]')]
      .some((e) => e.type !== 'hidden' && !!(e.offsetWidth || e.offsetHeight || e.getClientRects().length)),
    inputs: [...document.querySelectorAll('input')].map((e) => ({
      id: e.id || "",
      name: e.name || "",
      type: e.type || "",
      autocomplete: e.autocomplete || "",
      ariaLabel: e.getAttribute('aria-label') || "",
      placeholder: e.placeholder || "",
      visible: !!(e.offsetWidth || e.offsetHeight || e.getClientRects().length),
    })),
    buttons: [...document.querySelectorAll('button,a,[role="button"],input[type="submit"]')]
      .map((e) => (e.innerText || e.value || e.getAttribute('aria-label') || '').trim())
      .filter(Boolean)
      .slice(0, 120),
  }))()`);
}

function stateFingerprint(current) {
  const visibleInputs = (current.inputs || [])
    .filter((item) => item.visible)
    .map((item) => `${item.type}:${item.id}:${item.name}`)
    .join("|");
  return `${current.url}|${current.title}|${visibleInputs}`;
}

async function waitForStateChange(previous, timeoutMs = 12_000) {
  const fingerprint = stateFingerprint(previous);
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    await sleep(500);
    const current = await state();
    if (stateFingerprint(current) !== fingerprint) {
      await sleep(3000);
      return true;
    }
  }
  return false;
}

async function clickText(label) {
  return await evalJs(`(() => {
    const target = ${jsString(label)}.toLowerCase();
    const elements = [...document.querySelectorAll('button,a,[role="button"],input[type="submit"]')];
    const element = elements.find((e) =>
      (e.innerText || e.value || e.getAttribute('aria-label') || '').trim().toLowerCase() === target
    );
    if (!element || element.disabled || element.getAttribute('aria-disabled') === 'true') return false;
    element.scrollIntoView({ block: 'center' });
    element.click();
    return true;
  })()`);
}

async function clickTextContaining(fragment) {
  return await evalJs(`(() => {
    const target = ${jsString(fragment)}.toLowerCase();
    const elements = [...document.querySelectorAll('button,a,[role="button"],input[type="submit"]')];
    const element = elements.find((e) => {
      const text = (e.innerText || e.value || e.getAttribute('aria-label') || '').trim().toLowerCase();
      return text.includes(target) && !e.disabled && e.getAttribute('aria-disabled') !== 'true';
    });
    if (!element) return false;
    element.scrollIntoView({ block: 'center' });
    element.click();
    return true;
  })()`);
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

async function clickStructuralAction(selectors) {
  return await evalJs(`(() => {
    const selectors = ${jsString(selectors)};
    for (const selector of selectors) {
      const elements = [...document.querySelectorAll(selector)];
      const element = elements.find((candidate) =>
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

async function setVisiblePasswordByIndex(index, value) {
  return await evalJs(`(() => {
    const visible = [...document.querySelectorAll('input[type="password"]')]
      .filter((e) => !!(e.offsetWidth || e.offsetHeight || e.getClientRects().length));
    const element = visible[${index}];
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

async function submitNearest(selector) {
  return await evalJs(`(() => {
    const input = document.querySelector(${jsString(selector)});
    const form = input?.closest('form');
    const button = form?.querySelector('button[type="submit"],input[type="submit"],button');
    if (button && !button.disabled) {
      button.click();
      return true;
    }
    if (form) {
      form.requestSubmit();
      return true;
    }
    return false;
  })()`);
}

async function browserPageTargets() {
  return await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
}

async function targetForUrl(fragment) {
  const targets = await browserPageTargets();
  return targets.find((item) => item.type === "page" && item.url.includes(fragment));
}

async function openOrFindPage(fragment, url) {
  const existing = await targetForUrl(fragment);
  if (existing?.webSocketDebuggerUrl) {
    return existing;
  }
  await fetch(`http://127.0.0.1:${port}/json/new?${encodeURIComponent(url)}`, {
    method: "PUT",
  });
  const deadline = Date.now() + 15_000;
  while (Date.now() < deadline) {
    const target = await targetForUrl(fragment);
    if (target?.webSocketDebuggerUrl) {
      return target;
    }
    await sleep(500);
  }
  throw new Error(`Browser page did not open: ${url}`);
}

async function evaluateInTarget(target, expression) {
  const targetWs = new WebSocket(target.webSocketDebuggerUrl);
  let targetId = 0;
  const targetPending = new Map();
  targetWs.onmessage = (event) => {
    const message = JSON.parse(event.data);
    if (targetPending.has(message.id)) {
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
  await targetSend("Runtime.enable");
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

function extractTotpCodeFrom2faFunValues(values) {
  for (const value of values) {
    const text = String(value || "");
    const match = text.match(/^\s*(\d{6})\s*$/) || text.match(/\b(\d{6})\b/);
    if (match) {
      return match[1];
    }
  }
  return "";
}

function totpCodeForRemaining(values, remaining) {
  const code = extractTotpCodeFrom2faFunValues(values);
  return code && (!remaining || remaining >= 15) ? code : "";
}

async function codeFrom2faFun() {
  if (!totpSecret) {
    return "";
  }
  console.log("Browser helper: waiting for 2fa.fun controls");
  const target = await openOrFindPage("2fa.fun", "https://2fa.fun/");
  const readyDeadline = Date.now() + 15_000;
  let ready = false;
  while (Date.now() < readyDeadline) {
    ready = await evaluateInTarget(
      target,
      `(() => {
        const textarea = document.querySelector('#SECRET2FA,textarea[name="SECRET2FA"],textarea');
        const button = [...document.querySelectorAll('button,input[type="submit"],[role="button"]')]
          .find((e) => /获取验证码|验证码|get codes?|code/i.test((e.innerText || e.value || e.getAttribute('aria-label') || '').trim()));
        return !!textarea && !!button;
      })()`
    );
    if (ready) {
      break;
    }
    await sleep(500);
  }
  if (!ready) {
    console.log("Browser helper: 2fa.fun controls were not ready");
    return "";
  }
  console.log("Browser helper: submitting TOTP secret to 2fa.fun");
  const submitted = await evaluateInTarget(
    target,
    `(() => {
      const textarea = document.querySelector('#SECRET2FA,textarea[name="SECRET2FA"],textarea');
      if (!textarea) return false;
      textarea.focus();
      const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value')?.set;
      if (setter) setter.call(textarea, ${jsString(totpSecret)}); else textarea.value = ${jsString(totpSecret)};
      textarea.dispatchEvent(new Event('input', { bubbles: true }));
      textarea.dispatchEvent(new Event('change', { bubbles: true }));
      const button = [...document.querySelectorAll('button,input[type="submit"],[role="button"]')]
        .find((e) => /获取验证码|验证码|get codes?|code/i.test((e.innerText || e.value || e.getAttribute('aria-label') || '').trim()));
      if (!button) return false;
      button.click();
      return true;
    })()`
  );
  if (!submitted) {
    console.log("Browser helper: 2fa.fun form submission failed");
    return "";
  }
  const deadline = Date.now() + 35_000;
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
    const code = totpCodeForRemaining(
      otpState?.values || [],
      Number(otpState?.remaining || 0)
    );
    if (code) {
      console.log("Browser helper: obtained a 2fa.fun code with a safe validity window");
      return code;
    }
  }
  return "";
}

async function submitGoogleTotpCode(code) {
  if (!code) {
    return false;
  }
  const selector =
    'input#totpPin,input[name="totpPin"],input[autocomplete="one-time-code"],input[type="tel"]';
  const filled = await setInput(selector, code);
  if (!filled) {
    return false;
  }
  await sleep(1500);
  return await clickStructuralAction([
    "#totpNext button",
    "#totpNext",
    'button[jsname="LgbsSe"].nCP5yc',
    'button[type="submit"]',
    'input[type="submit"]',
  ]);
}

function isGoogleLoginPage(url, lower) {
  return (
    url.includes("accounts.google.com") &&
    (url.includes("/signin/") ||
      url.includes("/v3/signin") ||
      lower.includes("sign in with your google account") ||
      lower.includes("use your google account"))
  );
}

function isGoogleTotpPrompt(url, lower) {
  return (
    url.includes("accounts.google.com") &&
    (url.includes("/challenge/totp") ||
      lower.includes("google authenticator") ||
      lower.includes("verification code from the google authenticator app") ||
      lower.includes("enter the code from your authenticator app"))
  );
}

function requiresManualGoogleStep(url, lower) {
  if (!url.includes("accounts.google.com")) {
    return false;
  }
  return (
    lower.includes("2-step verification") ||
    lower.includes("verify it’s you") ||
    lower.includes("verify it's you") ||
    lower.includes("check your phone") ||
    lower.includes("tap yes") ||
    lower.includes("passkey") ||
    lower.includes("security key") ||
    lower.includes("captcha") ||
    lower.includes("unusual activity") ||
    lower.includes("account recovery") ||
    lower.includes("try another way")
  );
}

function hasNewPasswordForm(current) {
  const visible = (current.inputs || []).filter(
    (item) => item.visible && item.type === "password"
  );
  const descriptors = visible.map((item) =>
    `${item.id} ${item.name} ${item.autocomplete} ${item.ariaLabel} ${item.placeholder}`.toLowerCase()
  );
  return (
    visible.length >= 2 &&
    (descriptors.some((value) => value.includes("new")) ||
      descriptors.some((value) => value.includes("confirm")) ||
      (current.url || "").includes("myaccount.google.com/signinoptions/password"))
  );
}

function looksLikeSuccess(current, submittedPasswordChange) {
  if (!submittedPasswordChange) {
    return false;
  }
  const lower = (current.text || "").toLowerCase();
  const url = current.url || "";
  const explicit =
    lower.includes("password changed") ||
    lower.includes("password was changed") ||
    lower.includes("password has been changed") ||
    lower.includes("your password was changed");
  const securityPage =
    url.includes("myaccount.google.com/security") && !hasNewPasswordForm(current);
  return explicit || securityPage;
}

function languageActionForAriaLabels(labels) {
  const normalized = labels.map((label) => String(label || "").trim());
  if (normalized.some((label) => /^Edit language:\s*English(?:\s|\()/i.test(label))) {
    return "done";
  }
  if (normalized.some((label) => /^Save language:\s*English(?:\s|\()/i.test(label))) {
    return "promote-existing";
  }
  if (normalized.includes("Save your language selection")) {
    return "save-selection";
  }
  if (normalized.some((label) => label.startsWith("Edit language:"))) {
    return "edit-preferred";
  }
  return "wait";
}

async function languageButtonAriaLabels() {
  return await evalJs(`[...document.querySelectorAll('button')]
    .filter((e) => !!(e.offsetWidth || e.offsetHeight || e.getClientRects().length))
    .map((e) => e.getAttribute('aria-label') || '')
    .filter(Boolean)`);
}

async function clickButtonByAriaPrefix(prefix) {
  return await evalJs(`(() => {
    const prefix = ${jsString(prefix)};
    const element = [...document.querySelectorAll('button')].find((candidate) =>
      (candidate.getAttribute('aria-label') || '').startsWith(prefix) &&
      !!(candidate.offsetWidth || candidate.offsetHeight || candidate.getClientRects().length) &&
      !candidate.disabled &&
      candidate.getAttribute('aria-disabled') !== 'true'
    );
    if (!element) return false;
    element.scrollIntoView({ block: 'center' });
    element.click();
    return true;
  })()`);
}

async function clickVisibleExactText(text) {
  return await evalJs(`(() => {
    const text = ${jsString(text)};
    const elements = [...document.querySelectorAll('[role="option"],[role="menuitem"],button,[role="button"],li,div')]
      .filter((candidate) =>
        (candidate.innerText || '').trim() === text &&
        !!(candidate.offsetWidth || candidate.offsetHeight || candidate.getClientRects().length) &&
        !candidate.disabled &&
        candidate.getAttribute('aria-disabled') !== 'true'
      );
    const element = elements.find((candidate) =>
      ![...candidate.children].some((child) => (child.innerText || '').trim() === text)
    ) || elements.at(-1);
    if (!element) return false;
    element.scrollIntoView({ block: 'center' });
    element.click();
    return true;
  })()`);
}

async function waitForLanguageAction(timeoutMs = 20_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const action = languageActionForAriaLabels(await languageButtonAriaLabels());
    if (action !== "wait") {
      return action;
    }
    await sleep(500);
  }
  return "wait";
}

async function waitForEnglishPreferred(timeoutMs = 20_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (languageActionForAriaLabels(await languageButtonAriaLabels()) === "done") {
      return true;
    }
    await sleep(500);
  }
  return false;
}

async function ensureEnglishPreferredLanguage() {
  console.log("Browser helper: setting preferred Google language to English");
  await navigate(languageUrl);
  let action = await waitForLanguageAction();
  if (action === "done") {
    console.log("Browser helper: preferred Google language is English");
    return;
  }

  if (action === "promote-existing") {
    if (!(await clickButtonByAriaPrefix("Save language: English"))) {
      throw new Error("English language promotion control disappeared");
    }
  } else if (action === "edit-preferred") {
    if (!(await clickButtonByAriaPrefix("Edit language:"))) {
      throw new Error("Preferred-language edit control disappeared");
    }
    await sleep(1500);
    if (!(await clickVisibleExactText("English"))) {
      throw new Error("English was not available in the language picker");
    }
    await sleep(1500);
    await clickVisibleExactText("United States");
  } else {
    throw new Error("Google language settings did not become ready");
  }

  if (await waitForEnglishPreferred(5_000)) {
    console.log("Browser helper: preferred Google language is English");
    return;
  }

  action = await waitForLanguageAction(5_000);
  if (action === "save-selection") {
    if (!(await clickButtonByAriaPrefix("Save your language selection"))) {
      throw new Error("Language-selection save control disappeared");
    }
  }
  if (!(await waitForEnglishPreferred())) {
    throw new Error("Google did not confirm English as the preferred language");
  }
  console.log("Browser helper: preferred Google language is English");
}

export {
  extractTotpCodeFrom2faFunValues,
  hasNewPasswordForm,
  isGoogleLoginPage,
  isGoogleTotpPrompt,
  looksLikeSuccess,
  languageActionForAriaLabels,
  requiresManualGoogleStep,
  totpCodeForRemaining,
};

async function main() {
  process.exitCode = 1;
  if (!port) {
    throw new Error("GOOGLE_DEVTOOLS_PORT is required");
  }
  if (!googleEmail || !currentPassword || (!verificationOnly && !newPassword)) {
    throw new Error(
      "Google email and current password are required; new password is also required outside verification-only mode"
    );
  }

  await openWebsocket();
  await send("Runtime.enable");
  await send("Page.enable");

  const deadline = Date.now() + timeoutSeconds * 1000;
  let lastAction = "started";
  let lastManualNoticeAt = 0;
  let attemptedAccountSelection = false;
  let attemptedEmail = false;
  let attemptedCurrentPassword = false;
  let attemptedTotp = false;
  let attemptedVerificationChoices = false;
  let attemptedAuthenticatorSelection = false;
  let attemptedSettingsNavigation = false;
  let attemptedOpenPasswordForm = false;
  let attemptedPasswordForm = false;
  let submittedPasswordChange = false;

  while (Date.now() < deadline) {
    const current = await state();
    const text = current.text || "";
    const lower = text.toLowerCase();
    const url = current.url || "";
    const visiblePasswords = (current.inputs || []).filter(
      (item) => item.visible && item.type === "password"
    );

    if (
      verificationOnly &&
      url.includes("myaccount.google.com") &&
      !url.includes("accounts.google.com")
    ) {
      console.log("Browser helper: Google verification completed");
      ws.close();
      process.exitCode = 0;
      return;
    }

    if (looksLikeSuccess(current, submittedPasswordChange)) {
      if (setEnglishLanguage) {
        await ensureEnglishPreferredLanguage();
      }
      console.log("Browser helper: Google password change completed");
      ws.close();
      process.exitCode = 0;
      return;
    }

    if (lower.includes("choose an account") && !attemptedAccountSelection) {
      const selected =
        (await clickText(googleEmail)) || (await clickTextContaining(googleEmail));
      if (!selected) {
        await clickTextContaining("use another account");
      }
      attemptedAccountSelection = true;
      lastAction = selected ? "selected Google account" : "selected another account";
      await waitForStateChange(current);
      continue;
    }

    if (current.emailInput && !attemptedEmail) {
      const filled = await setInput(
        '#identifierId,input[type="email"],input[name="identifier"]',
        googleEmail
      );
      await sleep(1500);
      const submitted = await clickStructuralAction([
        "#identifierNext button",
        "#identifierNext",
        'button[jsname="LgbsSe"].nCP5yc',
        'button[type="submit"]',
      ]);
      attemptedEmail = true;
      lastAction = `submitted Google email filled=${filled} submitted=${submitted}`;
      console.log("Browser helper: submitted Google email");
      await waitForStateChange(current);
      continue;
    }

    if (
      isGoogleLoginPage(url, lower) &&
      visiblePasswords.length === 1 &&
      !hasNewPasswordForm(current) &&
      !attemptedCurrentPassword
    ) {
      const filled =
        (await setInput('input[name="Passwd"],input[autocomplete="current-password"]', currentPassword)) ||
        (await setVisiblePasswordByIndex(0, currentPassword));
      await sleep(1500);
      const submitted = await clickStructuralAction([
        "#passwordNext button",
        "#passwordNext",
        'button[jsname="LgbsSe"].nCP5yc',
        'button[type="submit"]',
      ]);
      attemptedCurrentPassword = true;
      lastAction = `submitted Google current password filled=${filled} submitted=${submitted}`;
      console.log("Browser helper: submitted Google current password");
      await waitForStateChange(current);
      continue;
    }

    if (isGoogleTotpPrompt(url, lower) && auto2faFun && totpSecret && !attemptedTotp) {
      console.log("Browser helper: Google authenticator prompt detected");
      const code = await codeFrom2faFun();
      attemptedTotp = true;
      if (await submitGoogleTotpCode(code)) {
        lastAction = "submitted Google authenticator code from 2fa.fun";
        console.log("Browser helper: submitted Google authenticator code from 2fa.fun");
        await waitForStateChange(current);
        continue;
      }
    }

    if (
      requiresManualGoogleStep(url, lower) &&
      auto2faFun &&
      totpSecret &&
      !attemptedVerificationChoices
    ) {
      const openedChoices = await clickTextContaining("try another way");
      attemptedVerificationChoices = true;
      if (openedChoices) {
        lastAction = "opened Google verification method choices";
        await waitForStateChange(current);
        continue;
      }
    }

    if (
      requiresManualGoogleStep(url, lower) &&
      auto2faFun &&
      totpSecret &&
      !attemptedAuthenticatorSelection
    ) {
      const selected = await clickTextContaining("authenticator");
      attemptedAuthenticatorSelection = true;
      if (selected) {
        lastAction = "selected Google Authenticator";
        await waitForStateChange(current);
        continue;
      }
    }

    if (requiresManualGoogleStep(url, lower)) {
      if (Date.now() - lastManualNoticeAt > 10_000) {
        console.log("Browser helper: manual Google verification detected; complete it in the browser");
        lastManualNoticeAt = Date.now();
      }
      lastAction = "waiting for manual Google verification";
      await sleep(1500);
      continue;
    }

    if (!verificationOnly && hasNewPasswordForm(current) && !attemptedPasswordForm) {
      attemptedPasswordForm = true;
      const first =
        (await setInput('input[autocomplete="new-password"]', newPassword)) ||
        (await setVisiblePasswordByIndex(0, newPassword));
      const second = await setVisiblePasswordByIndex(1, newPassword);
      if (!first || !second) {
        lastAction = `new password form incomplete first=${first} second=${second}`;
        await sleep(1000);
        continue;
      }
      await sleep(2000);
      const submitted = await clickStructuralAction([
        'form button[type="submit"]',
        'button[jsname="LgbsSe"].nCP5yc',
        'button[type="submit"]',
        'input[type="submit"]',
      ]);
      submittedPasswordChange = Boolean(submitted);
      lastAction = `submitted Google password change submitted=${submitted}`;
      console.log("Browser helper: submitted Google password change form");
      await waitForStateChange(current, 20_000);
      continue;
    }

    if (
      url.includes("myaccount.google.com") &&
      !url.includes("/signinoptions/password") &&
      !verificationOnly &&
      !submittedPasswordChange &&
      !verificationOnly &&
      !attemptedSettingsNavigation
    ) {
      attemptedSettingsNavigation = true;
      await navigate(settingsUrl);
      lastAction = "navigated to Google password settings";
      await waitForStateChange(current);
      continue;
    }

    if (
      !submittedPasswordChange &&
      !attemptedOpenPasswordForm &&
      (current.buttons || []).some((button) =>
        button.toLowerCase().includes("change password")
      )
    ) {
      attemptedOpenPasswordForm = true;
      await clickTextContaining("change password");
      lastAction = "opened Google password form";
      await waitForStateChange(current);
      continue;
    }

    await sleep(1000);
  }

  const finalState = await state();
  ws.close();
  throw new Error(
    `Browser helper timed out; lastAction=${lastAction}; title=${finalState.title}; url=${finalState.url}`
  );
}

if (isMain) {
  main().catch((error) => {
    console.error(error?.stack || error?.message || String(error));
    process.exit(1);
  });
}
