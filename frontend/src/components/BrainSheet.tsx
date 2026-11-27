import { useCallback, useEffect, useState } from 'react';
import { X } from 'lucide-react';
import { brainApi } from '@/lib/api';
import { useWorkspace } from '@/context/WorkspaceContext';
import { BrainEntryKind, BrainEntryStatus, type BrainEntry } from '../../../shared/types';

type Tab = 'proposed' | 'active';
type Draft = { id: string | null; kind: BrainEntryKind; title: string; body: string; everywhere: boolean };

const EMPTY: Draft = { id: null, kind: BrainEntryKind.decision, title: '', body: '', everywhere: false };

/**
 * The team's shared brain: agents propose, you approve. Approved entries are
 * sent to agents when they are relevant to a task, once per conversation.
 */
export function BrainSheet({ sessionId, onClose }: { sessionId: string; onClose: () => void }) {
  const { locale, showToast } = useWorkspace();
  const zh = locale === 'zh';
  const [tab, setTab] = useState<Tab>('proposed');
  const [entries, setEntries] = useState<BrainEntry[]>([]);
  const [proposedCount, setProposedCount] = useState(0);
  const [draft, setDraft] = useState<Draft | null>(null);
  const [busy, setBusy] = useState(false);

  const fail = useCallback((error: unknown) => showToast(error instanceof Error ? error.message : String(error)), [showToast]);
  const reload = useCallback(async () => {
    try {
      const [proposed, active] = await Promise.all([
        brainApi.list(sessionId, BrainEntryStatus.proposed),
        brainApi.list(sessionId, BrainEntryStatus.active),
      ]);
      setProposedCount(proposed.length);
      setEntries(tab === 'proposed' ? proposed : active);
    } catch (error) {
      fail(error);
    }
  }, [sessionId, tab, fail]);
  useEffect(() => { void reload(); }, [reload]);

  const act = async (work: () => Promise<unknown>, done?: string) => {
    setBusy(true);
    try {
      await work();
      if (done) showToast(done);
      await reload();
    } catch (error) {
      fail(error);
    } finally {
      setBusy(false);
    }
  };

  const save = () => {
    if (!draft) return;
    const { id, kind, title, body, everywhere } = draft;
    void act(async () => {
      if (id) {
        // Saving an edited proposal also approves it.
        await brainApi.update(id, { kind, title, body, status: BrainEntryStatus.active, supersedes_id: null });
      } else {
        await brainApi.create(sessionId, { kind, title, body, everywhere });
      }
      setDraft(null);
    }, zh ? '已记住' : 'Saved');
  };

  const kindLabel = (kind: BrainEntryKind) => ({
    decision: zh ? '决定' : 'Decision',
    rule: zh ? '规则' : 'Rule',
    preference: zh ? '偏好' : 'Preference',
    fact: zh ? '事实' : 'Fact',
  })[kind];

  return (
    <div className="ui-sheet-backdrop" role="dialog" aria-modal="true" aria-label={zh ? '共享大脑' : 'Shared brain'} onClick={onClose}>
      <div className="ui-sheet ui-sheet-wide" onClick={event => event.stopPropagation()}>
        <div className="ui-sheet-head">
          <strong>{zh ? '共享大脑' : 'Shared brain'}</strong>
          <button type="button" className="ui-sheet-close" onClick={onClose} aria-label={zh ? '关闭' : 'Close'}><X size={18} /></button>
        </div>
        <p className="ui-sheet-hint">
          {zh
            ? 'Agent 提出的建议，你批准后才会生效。生效的内容在相关任务里自动发给 Agent，同一段对话只发一次；同一个项目文件夹的所有群共用。'
            : 'Agents propose, you approve. Approved entries are sent to agents when relevant, once per conversation, and shared by every group in the same project folder.'}
        </p>
        <div className="ui-segment" role="tablist">
          <button type="button" role="tab" aria-checked={tab === 'proposed'} onClick={() => setTab('proposed')}>
            {zh ? '待批准' : 'Proposed'}{proposedCount > 0 ? ` (${proposedCount})` : ''}
          </button>
          <button type="button" role="tab" aria-checked={tab === 'active'} onClick={() => setTab('active')}>{zh ? '已生效' : 'In use'}</button>
          <button type="button" onClick={() => setDraft(EMPTY)}>{zh ? '+ 自己写一条' : '+ Add'}</button>
        </div>

        {draft && (
          <div className="ui-schedule-form" style={{ marginTop: 12 }}>
            <div className="ui-segment" role="radiogroup">
              {[BrainEntryKind.decision, BrainEntryKind.rule, BrainEntryKind.preference, BrainEntryKind.fact].map(kind => (
                <button key={kind} type="button" role="radio" aria-checked={draft.kind === kind} onClick={() => setDraft({ ...draft, kind })}>{kindLabel(kind)}</button>
              ))}
            </div>
            <input className="ui-input" placeholder={zh ? '一句话标题' : 'One-line title'} value={draft.title} onChange={event => setDraft({ ...draft, title: event.target.value })} />
            <textarea className="ui-input" rows={3} placeholder={zh ? '内容，例如「提交信息只写技术内容，不写工具名」' : 'Content'} value={draft.body} onChange={event => setDraft({ ...draft, body: event.target.value })} />
            {!draft.id && (
              <label className="ui-audit-filter">
                <input type="checkbox" checked={draft.everywhere} onChange={event => setDraft({ ...draft, everywhere: event.target.checked })} />
                {zh ? '所有群都用（不只这个项目）' : 'Use in every group, not only this project'}
              </label>
            )}
            <div className="ui-schedule-actions">
              <button type="button" className="ui-sheet-done" onClick={() => setDraft(null)}>{zh ? '取消' : 'Cancel'}</button>
              <button type="button" className="ui-sheet-primary" disabled={busy || !draft.title.trim() || !draft.body.trim()} onClick={save}>
                {draft.id ? (zh ? '保存并批准' : 'Save and approve') : (zh ? '保存' : 'Save')}
              </button>
            </div>
          </div>
        )}

        <ul className="ui-audit-list">
          {entries.map(entry => (
            <li key={entry.id}>
              <span className="ui-audit-meta">
                <span className="ui-audit-kind">{kindLabel(entry.kind)}</span>
                {entry.source_agent && <span>@{entry.source_agent}</span>}
                {entry.scope === '' && <span>{zh ? '所有群' : 'all groups'}</span>}
                {entry.locked && <span>{zh ? '已锁定' : 'locked'}</span>}
                {tab === 'active' && entry.use_count > 0 && <span>{zh ? `用过 ${entry.use_count} 次` : `used ${entry.use_count}×`}</span>}
              </span>
              <strong className="ui-brain-title">{entry.title}</strong>
              {entry.body !== entry.title && <span className="ui-brain-body">{entry.body}</span>}
              <span className="ui-schedule-actions">
                {tab === 'proposed' ? (
                  <>
                    <button type="button" className="ui-sheet-done" disabled={busy} onClick={() => void act(() => brainApi.update(entry.id, { kind: null, title: null, body: null, status: BrainEntryStatus.rejected, supersedes_id: null }))}>{zh ? '拒绝' : 'Reject'}</button>
                    <button type="button" className="ui-sheet-done" disabled={busy} onClick={() => setDraft({ id: entry.id, kind: entry.kind, title: entry.title, body: entry.body, everywhere: false })}>{zh ? '修改' : 'Edit'}</button>
                    <button type="button" className="ui-sheet-add" disabled={busy} onClick={() => void act(() => brainApi.update(entry.id, { kind: null, title: null, body: null, status: BrainEntryStatus.active, supersedes_id: null }), zh ? '已批准' : 'Approved')}>{zh ? '批准' : 'Approve'}</button>
                  </>
                ) : (
                  <>
                    <button type="button" className="ui-sheet-done" disabled={busy} onClick={() => setDraft({ id: entry.id, kind: entry.kind, title: entry.title, body: entry.body, everywhere: false })}>{zh ? '修改' : 'Edit'}</button>
                    <button type="button" className="ui-sheet-done" disabled={busy} onClick={() => void act(() => brainApi.remove(entry.id))}>{zh ? '删除' : 'Delete'}</button>
                  </>
                )}
              </span>
            </li>
          ))}
        </ul>
        {entries.length === 0 && (
          <p className="ui-sheet-hint">
            {tab === 'proposed'
              ? (zh ? '没有待批准的建议。Agent 在回复里写下决定或规则时，会出现在这里。' : 'Nothing to approve. When agents record a decision or rule, it shows up here.')
              : (zh ? '还没有生效的内容。' : 'Nothing in use yet.')}
          </p>
        )}
      </div>
    </div>
  );
}
