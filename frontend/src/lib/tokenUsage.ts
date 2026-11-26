/**
 * Per-run token usage, normalised across runtimes.
 *
 * Providers report prompt tokens differently: OpenAI-style runtimes (Codex)
 * include cached tokens in `input_tokens`, while Anthropic-style runtimes
 * (Claude Code) report cache reads separately. Both are mapped to the same
 * three numbers: fresh input, cached input and output.
 */
export interface TokenUsageSummary {
  freshInput: number;
  cachedInput: number;
  output: number;
  /** Share of prompt tokens served from cache, 0..1; null when unknown. */
  cacheHitRate: number | null;
  estimated: boolean;
}

const INPUT_INCLUDES_CACHE = new Set(['codex']);

const asCount = (value: unknown): number | null =>
  typeof value === 'number' && Number.isFinite(value) && value >= 0 ? value : null;

export const summarizeTokenUsage = (meta: unknown): TokenUsageSummary | null => {
  if (!meta || typeof meta !== 'object') return null;
  const usage = (meta as Record<string, unknown>).token_usage;
  if (!usage || typeof usage !== 'object') return null;
  const record = usage as Record<string, unknown>;
  const input = asCount(record.input_tokens);
  const output = asCount(record.output_tokens);
  if (input === null && output === null) return null;
  const cached = asCount(record.cache_read_tokens) ?? 0;
  const runtime = typeof record.runtime_agent === 'string' ? record.runtime_agent.toLowerCase() : '';
  const inclusive = INPUT_INCLUDES_CACHE.has(runtime) && (input ?? 0) >= cached;
  const freshInput = inclusive ? (input ?? 0) - cached : (input ?? 0);
  const promptTotal = freshInput + cached;
  return {
    freshInput,
    cachedInput: cached,
    output: output ?? 0,
    cacheHitRate: input === null || promptTotal === 0 ? null : cached / promptTotal,
    estimated: record.is_estimated === true,
  };
};

const compact = (count: number): string =>
  count >= 1000 ? `${(count / 1000).toFixed(count >= 10_000 ? 0 : 1)}k` : String(count);

/** One-line label such as `input 3.1k · cached 18k (85%) · output 900`. */
export const formatTokenUsage = (summary: TokenUsageSummary): string => {
  const prefix = summary.estimated ? '≈ ' : '';
  const rate =
    summary.cacheHitRate === null ? '' : ` (${Math.round(summary.cacheHitRate * 100)}%)`;
  return `${prefix}input ${compact(summary.freshInput)} · cached ${compact(summary.cachedInput)}${rate} · output ${compact(summary.output)}`;
};
