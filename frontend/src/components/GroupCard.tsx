import { useState } from 'react';
import { MessagesSquare } from 'lucide-react';
import type { Session } from '@/types';
import { copyPlainText } from '@/lib/clipboard';

export interface GroupCardProps {
  group: Session;
  onSelect: (id: string) => void;
  activityLabel: string;
  locale?: string;
}

export function GroupCard({ group, onSelect, activityLabel, locale }: GroupCardProps) {
  const [copyState, setCopyState] = useState<'idle' | 'copied' | 'failed'>('idle');
  const copy = async () => {
    const text = `Session: ${group.title}\nSession ID: ${group.id}\nGET /api/capabilities\nGET /api/chat/sessions/${encodeURIComponent(group.id)}/runtime`;
    setCopyState(await copyPlainText(text) ? 'copied' : 'failed');
  };
  return (
    <article className="ui-card">
      <div className="flex w-full items-center justify-between gap-2">
        <span className="ui-avatar">{Array.from(group.title.trim())[0] || '#'}</span>
        <button type="button" className="ui-copy" onClick={() => void copy()}>
          {copyState === 'copied' ? (locale === 'zh' ? '已复制' : 'Copied') : copyState === 'failed' ? (locale === 'zh' ? '复制失败，请重试' : 'Copy failed; retry') : (locale === 'zh' ? '复制给我的 Agent' : 'Copy for my agent')}
        </button>
      </div>
      <button type="button" className="ui-select" onClick={() => onSelect(group.id)} aria-current={group.active ? 'page' : undefined}>
        <span className="flex items-center gap-2"><MessagesSquare size={16} aria-hidden="true" /><span className="ui-card-title">{group.title}</span></span>
        <span className="ui-card-description">{locale === 'zh' ? '打开会话，继续查看任务、投递与协作记录。' : 'Open the conversation to continue tasks, deliveries and collaboration.'}</span>
        <span className="ui-card-meta">{group.hasRunningAgent ? activityLabel : (locale === 'zh' ? '会话' : 'Session')} · {group.id.slice(0, 8)}</span>
      </button>
    </article>
  );
}
