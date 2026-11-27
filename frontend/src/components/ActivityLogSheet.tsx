import { useCallback, useEffect, useState } from 'react';
import { X } from 'lucide-react';
import { chatAuditApi } from '@/lib/api';
import { useWorkspace } from '@/context/WorkspaceContext';
import type { ChatAuditEvent } from '../../../shared/types';

const PAGE = 50;

/** What agents in this group asked to do, what you decided, and undone runs. */
export function ActivityLogSheet({ sessionId, onClose }: { sessionId: string; onClose: () => void }) {
  const { locale, members, showToast } = useWorkspace();
  const zh = locale === 'zh';
  const [dangerousOnly, setDangerousOnly] = useState(false);
  const [events, setEvents] = useState<ChatAuditEvent[]>([]);
  const [more, setMore] = useState(false);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async (before?: ChatAuditEvent) => {
    setLoading(true);
    try {
      const page = await chatAuditApi.list(sessionId, { before, dangerous: dangerousOnly, limit: PAGE });
      setEvents(current => (before ? [...current, ...page] : page));
      setMore(page.length === PAGE);
    } catch (error) {
      showToast(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }, [sessionId, dangerousOnly, showToast]);

  useEffect(() => { void load(); }, [load]);

  const who = (event: ChatAuditEvent) =>
    members.find(member => member.id === event.session_agent_id)?.name ?? (zh ? '你' : 'You');
  const kind = (event: ChatAuditEvent) => {
    switch (event.kind) {
      case 'approval_requested': return zh ? '请求' : 'Asked';
      case 'approval_resolved': return zh ? '处理' : 'Decided';
      case 'run_undone': return zh ? '撤销' : 'Undone';
      default: return event.kind;
    }
  };
  const time = (value: string) =>
    new Intl.DateTimeFormat(locale, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(new Date(value));

  return (
    <div className="ui-sheet-backdrop" role="dialog" aria-modal="true" aria-label={zh ? '活动记录' : 'Activity log'} onClick={onClose}>
      <div className="ui-sheet ui-sheet-wide" onClick={event => event.stopPropagation()}>
        <div className="ui-sheet-head">
          <strong>{zh ? '活动记录' : 'Activity log'}</strong>
          <button type="button" className="ui-sheet-close" onClick={onClose} aria-label={zh ? '关闭' : 'Close'}><X size={18} /></button>
        </div>
        <p className="ui-sheet-hint">
          {zh ? 'Agent 请求过的操作、你的决定和撤销记录，保留 90 天。' : 'What agents asked to do, what you decided, and undone runs. Kept for 90 days.'}
        </p>
        <label className="ui-audit-filter">
          <input type="checkbox" checked={dangerousOnly} onChange={event => setDangerousOnly(event.target.checked)} />
          {zh ? '只看危险操作' : 'Dangerous only'}
        </label>
        <ol className="ui-audit-list">
          {events.map(event => (
            <li key={event.id} data-dangerous={event.dangerous || undefined}>
              <span className="ui-audit-meta">
                <time dateTime={event.created_at}>{time(event.created_at)}</time>
                <span>{who(event)}</span>
                <span className="ui-audit-kind">{kind(event)}</span>
                {event.dangerous && <span className="ui-audit-danger">{zh ? '危险' : 'Dangerous'}</span>}
              </span>
              <code>{event.summary}</code>
              {Array.isArray((event.detail as { reasons?: unknown }).reasons) && (event.detail as { reasons: string[] }).reasons.length > 0 && (
                <small>{(event.detail as { reasons: string[] }).reasons.join(' · ')}</small>
              )}
            </li>
          ))}
        </ol>
        {!loading && events.length === 0 && <p className="ui-sheet-hint">{zh ? '还没有记录。' : 'Nothing recorded yet.'}</p>}
        {more && (
          <button type="button" className="ui-sheet-done" disabled={loading} onClick={() => void load(events[events.length - 1])}>
            {loading ? '…' : (zh ? '更早的记录' : 'Older entries')}
          </button>
        )}
      </div>
    </div>
  );
}
