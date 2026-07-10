import assert from "node:assert/strict";

import {
  configuredDelayBounds,
  randomDelayMs,
} from "../scripts/github_step_delay.mjs";

const configured = configuredDelayBounds({
  KIRO_STEP_DELAY_MIN_MS: "2000",
  KIRO_STEP_DELAY_MAX_MS: "6000",
});
assert.deepEqual(configured, { minMs: 2000, maxMs: 6000 });
assert.equal(randomDelayMs(configured, () => 0), 2000);
assert.equal(randomDelayMs(configured, () => 0.999999), 6000);

const defaults = configuredDelayBounds({});
assert.deepEqual(defaults, { minMs: 2000, maxMs: 6000 });
assert.equal(randomDelayMs(defaults, () => 0), 2000);
assert.equal(randomDelayMs(defaults, () => 0.999999), 6000);

const disabled = configuredDelayBounds({
  KIRO_STEP_DELAY_MIN_MS: "0",
  KIRO_STEP_DELAY_MAX_MS: "0",
});
assert.deepEqual(disabled, { minMs: 0, maxMs: 0 });

console.log("github step delay tests passed");
