/** Presentation only: IDs sent to the backend remain unchanged. */
export function runtimeModelLabel(id: string): string {
  let value = id;
  try {
    const tuple = JSON.parse(id);
    if (Array.isArray(tuple) && tuple.length === 2 && tuple.every(part => typeof part === 'string')) value = tuple[1];
  } catch { /* Most executors use plain IDs. */ }
  const labels: Record<string, string> = {
    'deepseek-flash': 'DeepSeek Flash',
    'deepseek-v4-pro': 'DeepSeek V4 Pro',
    sonnet: 'Claude Sonnet', opus: 'Claude Opus', haiku: 'Claude Haiku',
  };
  return labels[value] ?? value;
}
