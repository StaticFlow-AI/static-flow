function parseDelayMs(value, fallback) {
  if (value === undefined || value === "") return fallback;
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 0) {
    throw new Error(`invalid non-negative delay: ${value}`);
  }
  return parsed;
}

export function configuredDelayBounds(env = process.env) {
  const minMs = parseDelayMs(env.KIRO_STEP_DELAY_MIN_MS, 2000);
  const maxMs = parseDelayMs(env.KIRO_STEP_DELAY_MAX_MS, Math.max(6000, minMs));
  if (maxMs < minMs) {
    throw new Error(`maximum delay ${maxMs} is below minimum delay ${minMs}`);
  }
  return { minMs, maxMs };
}

export function randomDelayMs({ minMs, maxMs }, random = Math.random) {
  if (maxMs === minMs) return minMs;
  return minMs + Math.floor(random() * (maxMs - minMs + 1));
}
