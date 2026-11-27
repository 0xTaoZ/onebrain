import { useEffect, useMemo, useState } from 'react';
import { Folder, MessageCircle, MessagesSquare, X } from 'lucide-react';
import { agentRuntimeApi, chatAgentsApi, chatSessionsApi, filesystemApi, sessionAgentsApi } from '@/lib/api';
import { useWorkspace } from '@/context/WorkspaceContext';
import { AgentCharacterAvatar } from './AgentCharacterAvatar';
import { handleFor, runtimeDisplayName } from './AddAgentSheet';

type Kind = 'direct' | 'chat' | 'project';
type Runtime = { runner: string; ready: boolean };

/**
 * Create a direct chat, a chat group or a project group in one step:
 * pick the type, tick the agents, and for a project choose its folder.
 */
export function NewGroupSheet({ onClose, onCreated }: { onClose: () => void; onCreated: (sessionId: string) => void }) {
  const { locale, showToast } = useWorkspace();
  const zh = locale === 'zh';
  const [kind, setKind] = useState<Kind>('chat');
  const [runtimes, setRuntimes] = useState<Runtime[] | null>(null);
  const [selected, setSelected] = useState<string[]>([]);
  const [folder, setFolder] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    agentRuntimeApi.list()
      .then(result => {
        const found = result.runners
          .filter(status => status.installed)
          .map(status => ({ runner: status.runner_type, ready: status.executable }));
        setRuntimes(found);
        setSelected(found.filter(item => item.ready).map(item => item.runner));
      })
      .catch(() => setRuntimes([]));
  }, []);

  const ready = useMemo(() => (runtimes ?? []).filter(item => item.ready), [runtimes]);

  const choose = (next: Kind) => {
    setKind(next);
    // A direct chat has exactly one agent; groups start with everyone who can run.
    setSelected(next === 'direct' ? ready.slice(0, 1).map(item => item.runner) : ready.map(item => item.runner));
  };

  const toggle = (runner: string) => {
    if (kind === 'direct') {
      setSelected([runner]);
      return;
    }
    setSelected(current => current.includes(runner) ? current.filter(item => item !== runner) : [...current, runner]);
  };

  const pickFolder = async () => {
    try {
      const result = await filesystemApi.selectDirectory({});
      if (!result.cancelled && result.path) setFolder(result.path);
    } catch (error) {
      showToast(error instanceof Error ? error.message : String(error));
    }
  };

  const title = (() => {
    const names = selected.map(runtimeDisplayName);
    if (kind === 'direct') return names[0] ?? (zh ? '单聊' : 'Direct chat');
    if (kind === 'project' && folder) return folder.split('/').filter(Boolean).pop() ?? folder;
    return names.slice(0, 3).join(zh ? '、' : ', ') + (names.length > 3 ? (zh ? ' 等' : ' and more') : '');
  })();

  const canCreate = selected.length > 0 && (kind !== 'project' || folder !== null) && !busy;

  const create = async () => {
    if (!canCreate) return;
    setBusy(true);
    try {
      const session = await chatSessionsApi.create({ title, workspace_path: kind === 'project' ? folder : null });
      const agents = await chatAgentsApi.list();
      for (const runner of selected) {
        const agent = agents.find(candidate => candidate.runner_type === runner) ?? await chatAgentsApi.create({
          name: handleFor(runner),
          runner_type: runner,
          system_prompt: null,
          tools_enabled: null,
          model_name: null,
        });
        await sessionAgentsApi.add(session.id, { agent_id: agent.id, workspace_path: null, allowed_skill_ids: null });
      }
      onCreated(session.id);
      onClose();
    } catch (error) {
      showToast(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  const kinds: Array<{ id: Kind; label: string; hint: string; Icon: typeof MessageCircle }> = [
    { id: 'direct', label: zh ? '单聊' : 'Direct', hint: zh ? '和一个 Agent 一对一' : 'One agent', Icon: MessageCircle },
    { id: 'chat', label: zh ? '聊天群' : 'Chat group', hint: zh ? '讨论、调研、写作' : 'Discuss and research', Icon: MessagesSquare },
    { id: 'project', label: zh ? '项目群' : 'Project group', hint: zh ? '在一个文件夹里写代码' : 'Code in a folder', Icon: Folder },
  ];

  return (
    <div className="ui-sheet-backdrop" role="dialog" aria-modal="true" aria-label={zh ? '新建' : 'New chat'} onClick={onClose}>
      <div className="ui-sheet" onClick={event => event.stopPropagation()}>
        <div className="ui-sheet-head">
          <strong>{zh ? '新建' : 'New chat'}</strong>
          <button type="button" className="ui-sheet-close" onClick={onClose} aria-label={zh ? '关闭' : 'Close'}><X size={18} /></button>
        </div>

        <div className="ui-kind-row" role="radiogroup">
          {kinds.map(({ id, label, hint, Icon }) => (
            <button key={id} type="button" role="radio" aria-checked={kind === id} className="ui-kind" onClick={() => choose(id)}>
              <Icon size={18} /><strong>{label}</strong><small>{hint}</small>
            </button>
          ))}
        </div>

        <p className="ui-sheet-label">{kind === 'direct' ? (zh ? '选一个成员' : 'Choose one agent') : (zh ? '选成员' : 'Choose agents')}</p>
        {runtimes === null ? (
          <p className="ui-sheet-hint">{zh ? '正在查找已安装的 Agent……' : 'Looking for installed agents…'}</p>
        ) : runtimes.length === 0 ? (
          <p className="ui-sheet-hint">
            {zh ? '没有找到编程 Agent。先在终端安装并登录 Claude Code、Codex 或 Gemini CLI。' : 'No coding agents found. Install and sign in to Claude Code, Codex or Gemini CLI first.'}
          </p>
        ) : (
          <div className="ui-pick-grid">
            {runtimes.map(({ runner, ready: runnable }) => (
              <button key={runner} type="button" className="ui-pick" aria-pressed={selected.includes(runner)} disabled={!runnable} onClick={() => toggle(runner)}>
                <span className="paper-person-avatar"><AgentCharacterAvatar identity={runner}>{runtimeDisplayName(runner)[0]}</AgentCharacterAvatar></span>
                <span>{runtimeDisplayName(runner)}</span>
                {!runnable && <small>{zh ? '需要登录' : 'Sign in first'}</small>}
              </button>
            ))}
          </div>
        )}

        {kind === 'project' && (
          <>
            <p className="ui-sheet-label">{zh ? '项目文件夹' : 'Project folder'}</p>
            <button type="button" className="ui-folder" onClick={() => void pickFolder()}>
              <Folder size={16} />{folder ?? (zh ? '选择文件夹…' : 'Choose a folder…')}
            </button>
          </>
        )}

        <button type="button" className="ui-sheet-primary" disabled={!canCreate} onClick={() => void create()}>
          {busy ? '…' : kind === 'direct' ? (zh ? '开始聊天' : 'Start chat') : (zh ? '建群' : 'Create group')}
        </button>
      </div>
    </div>
  );
}
