import assert from "node:assert/strict";
import test from "node:test";

import {
  extractTotpCodeFrom2faFunValues,
  isGithubTotpPrompt,
  isGoogleChallengeSelection,
  isGoogleTotpPrompt,
  isManualGoogleChallenge,
  selectAuthTarget,
  totpCodeForRemaining,
} from "../scripts/drive_kiro_github_via_google.mjs";

test("2fa.fun parsing accepts only a six-digit output with enough lifetime", () => {
  assert.equal(extractTotpCodeFrom2faFunValues([" 123456 "]), "123456");
  assert.equal(extractTotpCodeFrom2faFunValues(["secret", "12345"]), "");
  assert.equal(totpCodeForRemaining(["123456"], 14), "");
  assert.equal(totpCodeForRemaining(["123456"], 20), "123456");
});

test("Google and GitHub authenticator prompts are distinguished", () => {
  assert.equal(
    isGoogleTotpPrompt(
      "accounts.google.com",
      "/signin/challenge/totp",
      ["totpPin"],
      "Authenticator app"
    ),
    true
  );
  assert.equal(
    isGithubTotpPrompt(
      "github.com",
      "/sessions/two-factor/app",
      ["app_otp"],
      "Authentication code"
    ),
    true
  );
});

test("Google device-trust and recovery pages remain manual", () => {
  assert.equal(
    isManualGoogleChallenge(
      "accounts.google.com",
      "/signin/challenge/selection",
      "Use a device you have signed in on before"
    ),
    true
  );
});

test("Google challenge selection is handled before requesting a TOTP code", () => {
  assert.equal(
    isGoogleChallengeSelection(
      "accounts.google.com",
      "/v3/signin/challenge/selection"
    ),
    true
  );
  assert.equal(
    isGoogleTotpPrompt(
      "accounts.google.com",
      "/v3/signin/challenge/selection",
      [],
      "Get a verification code from the Google Authenticator app"
    ),
    false
  );
});

test("auth target selection ignores the 2fa.fun helper page", () => {
  const selected = selectAuthTarget([
    { id: "totp", type: "page", url: "https://2fa.fun/" },
    { id: "github", type: "page", url: "https://github.com/login" },
  ]);

  assert.equal(selected.id, "github");
});
