import { AgentProfileSheet } from './AgentProfileSheet';
import { useEffect, useState } from 'react';
import { X } from 'lucide-react';
import { agentRuntimeApi, chatAgentsApi, sessionAgentsApi } from '@/lib/api';
import { useWorkspace } from '@/context/WorkspaceContext';
import { AgentCharacterAvatar } from './AgentCharacterAvatar';

const NAMES: Record<string, string> = {
  CLAUDE_CODE: 'Claude Code',
  CODEX: 'Codex',
  GEMINI: 'Gemini CLI',
  QWEN_CODE: 'Qwen Code',
  OPENCODE: 'OpenCode',
  AMP: 'Amp',
  DEEPSEEK_HARNESS: 'DeepSeek Harness',
  CURSOR_AGENT: 'Cursor',
  COPILOT: 'Copilot',
  OPENTEAMS_CLI: 'OneBrain CLI',
  OPEN_TEAMS_CLI: 'OneBrain CLI',
  PI: 'Pi',
};

/** Human name for a runtime such as CLAUDE_CODE. */
export function runtimeDisplayName(runner: string): string {
  return NAMES[runner] ?? runner.toLowerCase().split('_').map(word => word[0]?.toUpperCase() + word.slice(1)).join(' ');
}

/** Short handle used for @mentions, such as "claude". */
export function handleFor(runner: string): string {
  return runtimeDisplayName(runner).split(' ')[0].toLowerCase();
}

type Runtime = { runner: string; ready: boolean };

/**
 * One-click way to bring installed coding agents into the current group.
 * Lists the agent CLIs found on this Mac; adding one creates the member and joins it.
 */
export function AddAgentSheet({ onClose, onCreateGroup }: { onClose: () => void; onCreateGroup: () => void }) {
  const { activeSessionId, members, refreshMembers, showToast, locale } = useWorkspace();
  const zh = locale === 'zh';
  const [runtimes, setRuntimes] = useState<Runtime[] | null>(null);
  const [editingMemberId, setEditingMemberId] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);

  useEffect(() => {
    agentRuntimeApi.list()
      .then(result => setRuntimes(result.runners
        .filter(status => status.installed)
        .map(status => ({ runner: status.runner_type, ready: status.executable }))))
      .catch(() => setRuntimes([]));
  }, []);

  const add = async (runner: string) => {
    if (!activeSessionId) return;
    setBusy(runner);
    try {
      const existing = (await chatAgentsApi.list()).find(agent => agent.runner_type === runner && agent.owner_project_id === null);
      const agent = existing ?? await chatAgentsApi.create({
        name: handleFor(runner),
        runner_type: runner,
        system_prompt: null,
        tools_enabled: null,
        model_name: null,
      });
      const member = await sessionAgentsApi.add(activeSessionId, { agent_id: agent.id, workspace_path: null, allowed_skill_ids: null });
      setEditingMemberId(member.id);
      await refreshMembers();
      showToast(zh ? `已加入：@${agent.name}` : `Added @${agent.name}`);
    } catch (error) {
      showToast(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(null);
    }
  };

  if (editingMemberId) return <AgentProfileSheet memberId={editingMemberId} onClose={() => setEditingMemberId(null)} />;

  return (
    <div className="ui-sheet-backdrop" role="dialog" aria-modal="true" aria-label={zh ? '加入成员' : 'Add agents'} onClick={onClose}>
      <div className="ui-sheet" onClick={event => event.stopPropagation()}>
        <div className="ui-sheet-head">
          <strong>{zh ? '加入成员' : 'Add agents'}</strong>
          <button type="button" className="ui-sheet-close" onClick={onClose} aria-label={zh ? '关闭' : 'Close'}><X size={18} /></button>
        </div>
        <p className="ui-sheet-hint">
          {zh
            ? '这些是你电脑上已经装好的编程 Agent。点「加入」，它就进群了，之后在聊天里用 @ 叫它。'
            : 'These coding agents are installed on this Mac. Click Add and it joins the group; then mention it with @ in the chat.'}
        </p>
        {!activeSessionId ? (
          <button type="button" className="ui-sheet-primary" onClick={() => { onClose(); onCreateGroup(); }}>
            {zh ? '先新建一个群' : 'Create a group first'}
          </button>
        ) : runtimes === null ? (
          <p className="ui-sheet-hint">{zh ? '正在查找……' : 'Looking for installed agents…'}</p>
        ) : runtimes.length === 0 ? (
          <p className="ui-sheet-hint">
            {zh
              ? '没有找到已安装的编程 Agent。先在终端安装并登录 Claude Code、Codex 或 Gemini CLI，再打开这里。'
              : 'No coding agents found. Install and sign in to Claude Code, Codex or Gemini CLI in Terminal, then open this again.'}
          </p>
        ) : (
          <ul className="ui-sheet-list">
            {runtimes.map(({ runner, ready }) => {
              const member = members.find(item => item.runner === runner);
              return (
                <li key={runner}>
                  <span className="paper-person-avatar"><AgentCharacterAvatar identity={runner}>{runtimeDisplayName(runner)[0]}</AgentCharacterAvatar></span>
                  <span className="ui-sheet-name">
                    <strong>{runtimeDisplayName(runner)}</strong>
                    <small>{ready ? `@${handleFor(runner)}` : (zh ? '已安装，但还不能运行' : 'Installed but not runnable yet')}</small>
                  </span>
                  {member ? (
                    <button type="button" className="ui-sheet-done" onClick={() => setEditingMemberId(member.id)}>{zh ? '设置' : 'Configure'}</button>
                  ) : (
                    <button type="button" className="ui-sheet-add" disabled={!ready || busy !== null} onClick={() => void add(runner)}>
                      {busy === runner ? '…' : (zh ? '加入' : 'Add')}
                    </button>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
}
