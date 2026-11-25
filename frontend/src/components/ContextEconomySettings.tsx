import { useState } from 'react';
import { useWorkspace } from '@/context/WorkspaceContext';

export function ContextEconomySettings() {
  const { config, locale, saveConfigPatch } = useWorkspace();
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const zh = locale === 'zh';
  const compression = config?.chat_compression;
  const enabled = compression?.allow_ai_summary ?? false;
  const toggle = async () => {
    if (!config || !compression || saving) return;
    setSaving(true);
    setError(null);
    try {
      await saveConfigPatch({ chat_compression: {
        ...compression,
        allow_ai_summary: !enabled,
      } });
    } catch {
      setError(zh ? '保存失败，设置未更改，请重试。' : 'Could not save. The setting was not changed; please retry.');
    } finally {
      setSaving(false);
    }
  };
  return <section className="space-y-2.5" aria-labelledby="context-economy-title">
    <h4 id="context-economy-title" className="settings-section-header">{zh ? '上下文与用量' : 'Context and usage'}</h4>
    <div className="settings-list-group rounded-lg border">
      <div className="settings-preference-row flex items-center justify-between gap-4 px-4 py-3.5">
        <div className="min-w-0">
          <p id="ai-summary-label" className="settings-row-title">{zh ? '允许 AI 整理长对话' : 'Allow AI summaries of long conversations'}</p>
          <p id="ai-summary-description" className="settings-row-description mt-1 leading-snug">{zh ? '默认关闭。开启后，长对话达到阈值时可调用现有 Agent 生成摘要，可能消耗额度。关闭时仍保留原始消息与按需读取，不额外调用摘要模型。' : 'Off by default. When enabled, long conversations may use a configured agent to generate summaries and consume quota. When off, original messages and on-demand reading remain available without extra summary calls.'}</p>
        </div>
        <button type="button" role="switch" aria-checked={enabled} aria-labelledby="ai-summary-label" aria-describedby="ai-summary-description" disabled={!compression || saving} onClick={() => void toggle()} className={`relative h-6 w-11 shrink-0 rounded-full transition-colors disabled:opacity-50 ${enabled ? 'bg-[var(--primary)]' : 'bg-[var(--surface-4)]'}`}>
          <span className={`absolute top-1 h-4 w-4 rounded-full bg-white shadow-sm transition-transform ${enabled ? 'left-6' : 'left-1'}`} />
        </button>
      </div>
      {error && <p role="alert" className="px-4 pb-3 text-xs text-red-600">{error}</p>}
    </div>
  </section>;
}
