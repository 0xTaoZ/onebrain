import { useCallback, useEffect, useMemo, useState } from 'react';
import { Brain, FileText, History, Search, Undo2, Wand2 } from 'lucide-react';
import { brainApi } from '@/lib/api';
import { notifyChatInputPrefill } from '@/lib/chatInputPrefill';
import { useWorkspace } from '@/context/WorkspaceContext';
import type { BrainChange, BrainFileContent, BrainOverview, BrainSearchHit } from '../../../shared/types';

/**
 * The team brain: a folder of Markdown that agents keep up to date. This page
 * only shows it: what every agent pays to read it, who changed what in which
 * run, and old versions to go back to. Changes are made by agents.
 */
export function BrainPage() {
  const { locale, showToast, activeSessionId } = useWorkspace();
  const zh = locale === 'zh';
  const [overview, setOverview] = useState<BrainOverview | null>(null);
  const [open, setOpen] = useState<{ path: string; commit?: string } | null>(null);
  const [file, setFile] = useState<BrainFileContent | null>(null);
  const [query, setQuery] = useState('');
  const [hits, setHits] = useState<BrainSearchHit[] | null>(null);

  const fail = useCallback((error: unknown) => showToast(error instanceof Error ? error.message : String(error)), [showToast]);
  const reload = useCallback(() => brainApi.overview().then(setOverview).catch(fail), [fail]);
  useEffect(() => { void reload(); }, [reload]);

  useEffect(() => {
    if (!open) { setFile(null); return; }
    brainApi.file(open.path, open.commit).then(setFile).catch(fail);
  }, [open, fail]);

  useEffect(() => {
    const q = query.trim();
    if (!q) { setHits(null); return; }
    const timer = window.setTimeout(() => brainApi.search(q).then(setHits).catch(fail), 200);
    return () => window.clearTimeout(timer);
  }, [query, fail]);

  const time = useMemo(
    () => new Intl.DateTimeFormat(locale, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }),
    [locale],
  );
  const when = (at?: string | null) => (at ? time.format(new Date(at)) : '');
  const tokens = (n: number) => (n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n));

  const askAgent = (path: string) => {
    if (!activeSessionId) {
      showToast(zh ? '先打开一个群' : 'Open a group first');
      return;
    }
    // Go back to the chat first so its composer is there to receive the text.
    window.dispatchEvent(new CustomEvent('openteams:navigate-session', { detail: activeSessionId }));
    window.setTimeout(() => notifyChatInputPrefill({
      sessionId: activeSessionId,
      text: zh ? `请更新共享大脑里的 ${path}：` : `Please update ${path} in the team brain: `,
      mode: 'free',
    }), 150);
  };

  const restore = async (path: string, commit: string) => {
    try {
      await brainApi.restore(path, commit);
      showToast(zh ? '已恢复，这次恢复也记在历史里' : 'Restored; the restore is in the history too');
      setOpen({ path });
      void reload();
    } catch (error) {
      fail(error);
    }
  };

  const groups = useMemo(() => {
    const map = new Map<string, BrainOverview['files']>();
    for (const item of overview?.files ?? []) {
      const top = item.always_read ? (zh ? '每次必读' : 'Always read') : item.path.includes('/') ? item.path.split('/')[0] : (zh ? '根目录' : 'Top level');
      map.set(top, [...(map.get(top) ?? []), item]);
    }
    return [...map.entries()];
  }, [overview, zh]);

  const change = (item: BrainChange) => (
    <li key={item.commit} className="ui-brain-change">
      <span className="ui-entry-meta">
        <strong>{item.author}</strong>
        <span>{when(item.at)}</span>
      </span>
      <span className="ui-brain-change-msg">{item.message}</span>
      <span className="ui-brain-change-files">
        {item.files.map(path => (
          <button key={path} type="button" className="ui-link" onClick={() => setOpen({ path })}>{path}</button>
        ))}
      </span>
    </li>
  );

  return (
    <div className="ui-page ui-brain">
      <div className="ui-page-head">
        <div>
          <h1><Brain size={20} />{zh ? '共享大脑' : 'Team brain'}</h1>
          <p>
            {zh
              ? '所有 Agent 共用的一份 Markdown 记忆，由 Agent 自己维护；Claude Code、Codex、Gemini 不经过 OneBrain 也能直接读。每次改动都记下是哪个 Agent 写的，可以随时退回。'
              : 'One Markdown memory shared by every agent and kept up to date by them; Claude Code, Codex and Gemini can read it without OneBrain. Every change records which agent made it and can be undone.'}
          </p>
          {overview && <code className="ui-brain-path">{overview.dir}</code>}
        </div>
      </div>

      {overview && (
        <div className="ui-brain-stats">
          <div className="ui-panel"><strong>{tokens(overview.always_read_tokens)}</strong><span>{zh ? '每个 Agent 每段对话必读的 token' : 'tokens every agent reads per conversation'}</span></div>
          <div className="ui-panel"><strong>{overview.files.length}</strong><span>{zh ? '个文件' : 'files'}</span></div>
          <div className="ui-panel"><strong>{tokens(overview.total_tokens)}</strong><span>{zh ? '全部内容的 token（按需读取）' : 'tokens in total, read on demand'}</span></div>
          <div className="ui-panel"><strong>{overview.recent.length}</strong><span>{zh ? '次最近改动' : 'recent changes'}</span></div>
        </div>
      )}

      <div className="ui-brain-body">
        <aside className="ui-panel ui-brain-files">
          <label className="ui-search">
            <Search size={14} />
            <input value={query} onChange={event => setQuery(event.target.value)} placeholder={zh ? '搜索全部文件' : 'Search all files'} />
          </label>
          {hits ? (
            <ul className="ui-brain-list">
              {hits.map(hit => (
                <li key={`${hit.path}:${hit.line}`}>
                  <button type="button" onClick={() => setOpen({ path: hit.path })}>
                    <span className="ui-brain-file-name">{hit.path}<small>:{hit.line}</small></span>
                    <small className="ui-brain-hit">{hit.text}</small>
                  </button>
                </li>
              ))}
              {hits.length === 0 && <li className="ui-page-empty">{zh ? '没有找到' : 'No matches'}</li>}
            </ul>
          ) : (
            groups.map(([group, items]) => (
              <div key={group} className="ui-brain-group">
                <p className="ui-brain-group-title">{group}</p>
                <ul className="ui-brain-list">
                  {items.map(item => (
                    <li key={item.path}>
                      <button type="button" aria-current={open?.path === item.path || undefined} onClick={() => setOpen({ path: item.path })}>
                        <span className="ui-brain-file-name"><FileText size={13} />{item.path}</span>
                        <small>{tokens(item.tokens)} tok{item.last_author ? ` · ${item.last_author}` : ''}</small>
                      </button>
                    </li>
                  ))}
                </ul>
              </div>
            ))
          )}
        </aside>

        <section className="ui-panel ui-brain-view">
          {open && file ? (
            <>
              <div className="ui-brain-view-head">
                <div>
                  <strong>{file.path}</strong>
                  <small>{open.commit ? (zh ? `旧版本 ${open.commit.slice(0, 8)}` : `version ${open.commit.slice(0, 8)}`) : (zh ? '当前版本' : 'current')}</small>
                </div>
                <div className="ui-row-end">
                  {open.commit ? (
                    <>
                      <button type="button" className="ui-btn" onClick={() => setOpen({ path: open.path })}>{zh ? '看当前版本' : 'Current'}</button>
                      <button type="button" className="ui-btn ui-btn-primary" onClick={() => void restore(open.path, open.commit!)}><Undo2 size={14} />{zh ? '恢复到这个版本' : 'Restore this version'}</button>
                    </>
                  ) : (
                    <button type="button" className="ui-btn ui-btn-primary" onClick={() => askAgent(file.path)}><Wand2 size={14} />{zh ? '让 Agent 修改' : 'Ask an agent to change it'}</button>
                  )}
                </div>
              </div>
              <pre className="ui-brain-text">{file.text}</pre>
              <p className="ui-brain-group-title"><History size={13} />{zh ? '这个文件的改动' : 'Changes to this file'}</p>
              <ul className="ui-brain-history">
                {file.history.map(item => (
                  <li key={item.commit}>
                    <button type="button" aria-current={open.commit === item.commit || undefined} onClick={() => setOpen({ path: file.path, commit: item.commit })}>
                      <strong>{item.author}</strong>
                      <span>{when(item.at)}</span>
                      <span className="ui-brain-change-msg">{item.message}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </>
          ) : (
            <>
              <p className="ui-brain-group-title"><History size={13} />{zh ? '最近改动' : 'Recent changes'}</p>
              <ul className="ui-brain-changes">{overview?.recent.map(change)}</ul>
              {overview && overview.recent.length === 0 && <p className="ui-page-empty">{zh ? '还没有改动。' : 'No changes yet.'}</p>}
            </>
          )}
        </section>
      </div>
    </div>
  );
}
