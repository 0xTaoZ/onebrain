import assert from 'node:assert/strict';
import { formatTokenUsage, summarizeTokenUsage } from './tokenUsage';

// Claude Code reports cache reads separately from input.
const claude = summarizeTokenUsage({
  token_usage: { runtime_agent: 'claude_code', input_tokens: 3000, cache_read_tokens: 17000, output_tokens: 900 },
});
assert.deepEqual(claude, { freshInput: 3000, cachedInput: 17000, output: 900, cacheHitRate: 0.85, estimated: false });
assert.equal(formatTokenUsage(claude!), 'input 3.0k · cached 17k (85%) · output 900');

// Codex includes cached tokens in input.
const codex = summarizeTokenUsage({
  token_usage: { runtime_agent: 'codex', input_tokens: 20000, cache_read_tokens: 15000, output_tokens: 1200 },
});
assert.equal(codex!.freshInput, 5000);
assert.equal(codex!.cacheHitRate, 0.75);

// Missing data stays unknown instead of becoming a fake 0%.
assert.equal(summarizeTokenUsage({}), null);
assert.equal(summarizeTokenUsage({ token_usage: { output_tokens: 10 } })!.cacheHitRate, null);
assert.match(
  formatTokenUsage(summarizeTokenUsage({ token_usage: { input_tokens: 100, output_tokens: 5, is_estimated: true } })!),
  /^≈ /,
);

console.log('tokenUsage tests passed');
