import { useCallback, useEffect, useMemo, useState } from 'react';
import { Brain, Lock, Plus, Search } from 'lucide-react';
import { brainApi } from '@/lib/api';
import { useWorkspace } from '@/context/WorkspaceContext';
import { BrainEntryKind, BrainEntryStatus, type BrainEntry } from '../../../shared/types';

type Tab = 'proposed' | 'active' | 'history';
type Draft = { id: string | null; kind: BrainEntryKind; title: string; body: string; everywhere: boolean };

const KINDS = [BrainEntryKind.decision, BrainEntryKind.rule, BrainEntryKind.preference, BrainEntryKind.fact];
const EMPTY: Draft = { id: null, kind: BrainEntryKind.decision, title: '', body: '', everywhere: false };

/**
 * The team's shared brain as a page: what agents proposed, what is in use,
 * and what was replaced or rejected. Agents propose, the user approves.
 */
export function BrainPage() {
  const { locale, showToast, activeSessionId, sessions } = useWorkspace();
  const zh = locale === 'zh';
  const [tab, setTab] = useState<Tab>('proposed');
  const [lists, setLists] = useState<Record<Tab, BrainEntry[]>>({ proposed: [], active: [], history: [] });
  const [query, setQuery] = useState('');
  const [kind, setKind] = useState<BrainEntryKind | 'all'>('all');
  const [draft, setDraft] = useState<Draft | null>(null);
  const [busy, setBusy] = useState(false);

  const fail = useCallback((error: unknown) => showToast(error instanceof Error ? error.message : String(error)), [showToast]);
  const reload = useCallback(async () => {
    if (!activeSessionId) return;
    try {
      const [proposed, active, superseded, rejected] = await Promise.all([
        brainApi.list(activeSessionId, BrainEntryStatus.proposed),
        brainApi.list(activeSessionId, BrainEntryStatus.active),
        brainApi.list(activeSessionId, BrainEntryStatus.superseded),
        brainApi.list(activeSessionId, BrainEntryStatus.rejected),
      ]);
      setLists({ proposed, active, history: [...superseded, ...rejected] });
    } catch (error) {
      fail(error);
    }
  }, [activeSessionId, fail]);
  useEffect(() => { void reload(); }, [reload]);
  // Land on what needs attention: proposals if any, otherwise what is in use.
  useEffect(() => {
    if (lists.proposed.length === 0 && tab === 'proposed' && lists.active.length > 0) setTab('active');
  }, [lists, tab]);

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

  const setStatus = (entry: BrainEntry, status: BrainEntryStatus, done?: string) =>
    act(() => brainApi.update(entry.id, { kind: null, title: null, body: null, status, supersedes_id: null }), done);

  const save = () => {
    if (!draft || !activeSessionId) return;
    const { id, kind: draftKind, title, body, everywhere } = draft;
    void act(async () => {
      if (id) {
        await brainApi.update(id, { kind: draftKind, title, body, status: BrainEntryStatus.active, supersedes_id: null });
      } else {
        await brainApi.create(activeSessionId, { kind: draftKind, title, body, everywhere });
      }
      setDraft(null);
    }, zh ? '已记住' : 'Saved');
  };

  const kindLabel = (value: BrainEntryKind) => ({
    decision: zh ? '决定' : 'Decision',
    rule: zh ? '规则' : 'Rule',
    preference: zh ? '偏好' : 'Preference',
    fact: zh ? '事实' : 'Fact',
  })[value];
  const sessionTitle = (id: string | null) => sessions.find(session => session.id === id)?.title;

  const shown = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return lists[tab].filter(entry =>
      (kind === 'all' || entry.kind === kind) &&
      (!needle || entry.title.toLowerCase().includes(needle) || entry.body.toLowerCase().includes(needle)));
  }, [lists, tab, kind, query]);

  if (!activeSessionId) {
    return <div className="ui-page"><p className="ui-page-empty">{zh ? '先打开一个群。' : 'Open a group first.'}</p></div>;
  }

  const tabs: Array<[Tab, string]> = [
    ['proposed', zh ? '待批准' : 'Proposed'],
    ['active', zh ? '已生效' : 'In use'],
    ['history', zh ? '历史' : 'History'],
  ];

  return (
    <div className="ui-page">
      <div className="ui-page-head">
        <div>
          <h1><Brain size={20} />{zh ? '共享大脑' : 'Shared brain'}</h1>
          <p>
            {zh
              ? 'Agent 提出的决定、规则和偏好，你批准后才会生效。生效的内容在相关任务里自动发给 Agent，同一段对话只发一次；同一个项目文件夹的所有群共用。'
              : 'Decisions, rules and preferences agents propose take effect only after you approve them. They are sent to agents when relevant, once per conversation, and shared by every group in the same project folder.'}
          </p>
        </div>
        <button type="button" className="ui-btn ui-btn-primary" onClick={() => setDraft(EMPTY)}><Plus size={16} />{zh ? '写一条' : 'Add'}</button>
      </div>

      <div className="ui-page-toolbar">
        <div className="ui-segment" role="tablist">
          {tabs.map(([id, label]) => (
            <button key={id} type="button" role="tab" aria-selected={tab === id} onClick={() => setTab(id)}>
              {label}<span className="ui-count">{lists[id].length}</span>
            </button>
          ))}
        </div>
        <label className="ui-search">
          <Search size={14} />
          <input value={query} onChange={event => setQuery(event.target.value)} placeholder={zh ? '搜索' : 'Search'} />
        </label>
        <select className="ui-input ui-select" value={kind} onChange={event => setKind(event.target.value as BrainEntryKind | 'all')}>
          <option value="all">{zh ? '全部类别' : 'All kinds'}</option>
          {KINDS.map(value => <option key={value} value={value}>{kindLabel(value)}</option>)}
        </select>
      </div>

      {draft && (
        <div className="ui-panel ui-editor">
          <div className="ui-segment" role="radiogroup">
            {KINDS.map(value => (
              <button key={value} type="button" role="radio" aria-checked={draft.kind === value} onClick={() => setDraft({ ...draft, kind: value })}>{kindLabel(value)}</button>
            ))}
          </div>
          <input className="ui-input" placeholder={zh ? '一句话标题' : 'One-line title'} value={draft.title} onChange={event => setDraft({ ...draft, title: event.target.value })} autoFocus />
          <textarea className="ui-input" rows={4} placeholder={zh ? '内容，例如「提交信息只写技术内容」' : 'Content'} value={draft.body} onChange={event => setDraft({ ...draft, body: event.target.value })} />
          {!draft.id && (
            <label className="ui-check">
              <input type="checkbox" checked={draft.everywhere} onChange={event => setDraft({ ...draft, everywhere: event.target.checked })} />
              {zh ? '所有群都用，不只这个项目' : 'Use in every group, not only this project'}
            </label>
          )}
          <div className="ui-row-end">
            <button type="button" className="ui-btn" onClick={() => setDraft(null)}>{zh ? '取消' : 'Cancel'}</button>
            <button type="button" className="ui-btn ui-btn-primary" disabled={busy || !draft.title.trim() || !draft.body.trim()} onClick={save}>
              {draft.id ? (zh ? '保存并生效' : 'Save and use') : (zh ? '保存' : 'Save')}
            </button>
          </div>
        </div>
      )}

      <ul className="ui-entry-list">
        {shown.map(entry => (
          <li key={entry.id} className="ui-panel ui-entry">
            <div className="ui-entry-meta">
              <span className="ui-tag">{kindLabel(entry.kind)}</span>
              {entry.source_agent && <span>{zh ? `来自 @${entry.source_agent}` : `from @${entry.source_agent}`}</span>}
              {sessionTitle(entry.source_session_id) && <span>{zh ? `在「${sessionTitle(entry.source_session_id)}」` : `in ${sessionTitle(entry.source_session_id)}`}</span>}
              {entry.scope === '' && <span>{zh ? '所有群' : 'all groups'}</span>}
              {entry.locked && <span className="ui-locked"><Lock size={12} />{zh ? '你改过，已锁定' : 'edited by you'}</span>}
              {tab === 'active' && entry.use_count > 0 && <span>{zh ? `用过 ${entry.use_count} 次` : `used ${entry.use_count}×`}</span>}
              {tab === 'history' && <span>{entry.status === 'superseded' ? (zh ? '已被替换' : 'replaced') : (zh ? '已拒绝' : 'rejected')}</span>}
            </div>
            <strong className="ui-entry-title">{entry.title}</strong>
            {entry.body !== entry.title && <p className="ui-entry-body">{entry.body}</p>}
            <div className="ui-row-end">
              {tab === 'proposed' && (
                <>
                  <button type="button" className="ui-btn" disabled={busy} onClick={() => void setStatus(entry, BrainEntryStatus.rejected)}>{zh ? '拒绝' : 'Reject'}</button>
                  <button type="button" className="ui-btn" disabled={busy} onClick={() => setDraft({ id: entry.id, kind: entry.kind, title: entry.title, body: entry.body, everywhere: false })}>{zh ? '修改' : 'Edit'}</button>
                  <button type="button" className="ui-btn ui-btn-primary" disabled={busy} onClick={() => void setStatus(entry, BrainEntryStatus.active, zh ? '已批准' : 'Approved')}>{zh ? '批准' : 'Approve'}</button>
                </>
              )}
              {tab === 'active' && (
                <>
                  <button type="button" className="ui-btn" disabled={busy} onClick={() => setDraft({ id: entry.id, kind: entry.kind, title: entry.title, body: entry.body, everywhere: false })}>{zh ? '修改' : 'Edit'}</button>
                  <button type="button" className="ui-btn" disabled={busy} onClick={() => void act(() => brainApi.remove(entry.id))}>{zh ? '删除' : 'Delete'}</button>
                </>
              )}
              {tab === 'history' && (
                <button type="button" className="ui-btn" disabled={busy} onClick={() => void setStatus(entry, BrainEntryStatus.active, zh ? '已恢复' : 'Restored')}>{zh ? '恢复使用' : 'Use again'}</button>
              )}
            </div>
          </li>
        ))}
      </ul>
      {shown.length === 0 && (
        <p className="ui-page-empty">
          {query || kind !== 'all'
            ? (zh ? '没有符合条件的内容。' : 'Nothing matches.')
            : tab === 'proposed'
              ? (zh ? '没有待批准的建议。Agent 写下决定或规则时会出现在这里。' : 'Nothing to approve. When agents record a decision or rule, it shows up here.')
              : tab === 'active'
                ? (zh ? '还没有生效的内容。' : 'Nothing in use yet.')
                : (zh ? '没有历史记录。' : 'No history.')}
        </p>
      )}
    </div>
  );
}
