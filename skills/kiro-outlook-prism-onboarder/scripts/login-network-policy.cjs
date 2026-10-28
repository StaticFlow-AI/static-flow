const DEFAULT_PROXY = 'http://127.0.0.1:11115';

function loginProxyForPosition(position, { blockSize = 3, proxy = DEFAULT_PROXY } = {}) {
  if (!Number.isInteger(position) || position < 1) throw new Error('Account position must be a positive integer');
  if (!Number.isInteger(blockSize) || blockSize < 1) throw new Error('Block size must be a positive integer');
  const url = new URL(proxy);
  if (url.protocol !== 'http:' || !['127.0.0.1', 'localhost', '[::1]'].includes(url.hostname) || url.username || url.password)
    throw new Error('Expected an unauthenticated local HTTP proxy');
  return Math.floor((position - 1) / blockSize) % 2 ? proxy : null;
}

function policyForBatch(count, options) {
  if (!Number.isInteger(count) || count < 1) throw new Error('Batch size must be a positive integer');
  return Array.from({ length: count }, (_, index) => ({
    batch_index: index + 1,
    login_proxy: loginProxyForPosition(index + 1, options),
  }));
}

if (require.main === module) {
  const args = process.argv.slice(2);
  if (args.length !== 2 || args[0] !== '--count') throw new Error('Usage: node login-network-policy.cjs --count NUMBER');
  console.log(JSON.stringify(policyForBatch(Number(args[1]))));
}

module.exports = { loginProxyForPosition, policyForBatch };
