import assert from "node:assert/strict";
import test from "node:test";

import {
  extractTotpCodeFrom2faFunValues,
  hasNewPasswordForm,
  isGoogleLoginPage,
  isGoogleTotpPrompt,
  languageActionForAriaLabels,
  looksLikeSuccess,
  requiresManualGoogleStep,
  totpCodeForRemaining,
} from "../scripts/drive_google_password_change.mjs";

test("extractTotpCodeFrom2faFunValues reads only six-digit values", () => {
  assert.equal(extractTotpCodeFrom2faFunValues(["", " 123456 "]), "123456");
  assert.equal(extractTotpCodeFrom2faFunValues(["secret", "12345"]), "");
});

test("2fa.fun codes are used only with a safe validity window", () => {
  assert.equal(totpCodeForRemaining(["123456"], 8), "");
  assert.equal(totpCodeForRemaining(["123456"], 20), "123456");
});

test("Google login and authenticator pages are detected", () => {
  assert.equal(
    isGoogleLoginPage(
      "https://accounts.google.com/v3/signin/identifier",
      "use your google account"
    ),
    true
  );
  assert.equal(
    isGoogleTotpPrompt(
      "https://accounts.google.com/signin/v2/challenge/totp",
      "enter the code from your authenticator app"
    ),
    true
  );
});

test("manual Google verification pages are detected", () => {
  assert.equal(
    requiresManualGoogleStep(
      "https://accounts.google.com/signin/v2/challenge/pk",
      "2-step verification check your phone"
    ),
    true
  );
  assert.equal(
    requiresManualGoogleStep("https://myaccount.google.com/security", "security"),
    false
  );
});

test("new-password form requires two visible password fields", () => {
  const current = {
    url: "https://myaccount.google.com/signinoptions/password",
    inputs: [
      { type: "password", visible: true, autocomplete: "new-password" },
      { type: "password", visible: true, name: "confirmation_password" },
    ],
  };
  assert.equal(hasNewPasswordForm(current), true);
});

test("success requires a submitted form and an explicit or security state", () => {
  const current = {
    url: "https://myaccount.google.com/security",
    text: "Password changed",
    inputs: [],
  };
  assert.equal(looksLikeSuccess(current, false), false);
  assert.equal(looksLikeSuccess(current, true), true);
});

test("preferred-language controls produce deterministic actions", () => {
  assert.equal(
    languageActionForAriaLabels(["Edit language: English (United States)"]),
    "done"
  );
  assert.equal(
    languageActionForAriaLabels(["Save language: English (English)"]),
    "promote-existing"
  );
  assert.equal(
    languageActionForAriaLabels(["Edit language: Tiếng Việt (Việt Nam)"]),
    "edit-preferred"
  );
  assert.equal(
    languageActionForAriaLabels(["Save your language selection"]),
    "save-selection"
  );
});
