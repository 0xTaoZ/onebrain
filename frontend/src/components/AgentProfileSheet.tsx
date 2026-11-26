import { useEffect, useId, useState } from 'react';
import { X } from 'lucide-react';
import { useWorkspace } from '@/context/WorkspaceContext';
import { agentRuntimeApi, chatAgentsApi, chatSessionsApi, projectApi, sessionAgentsApi } from '@/lib/api';
import type { BackendChatAgent } from '@/types';
import type { ProjectMemberWithRuntime } from '../../../shared/types';

export function AgentProfileSheet({ memberId, onClose }: { memberId: string; onClose: () => void }) {
  const { activeSessionId, locale, refreshMembers } = useWorkspace();
  const zh = locale === 'zh';
  const listId = useId();
  const [profile, setProfile] = useState<{ agent: BackendChatAgent; projectMember?: ProjectMemberWithRuntime }>();
  const [models, setModels] = useState<string[]>([]);
  const [model, setModel] = useState('');
  const [role, setRole] = useState('');
  const [error, setError] = useState('');
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let active = true;
    void (async () => {
      try {
        const [session, members] = await Promise.all([
          chatSessionsApi.get(activeSessionId),
          sessionAgentsApi.list(activeSessionId),
        ]);
        const member = members.find(item => item.id === memberId);
        if (!member) throw new Error(zh ? '找不到群成员' : 'Group member not found');
        const [agent, projectMembers] = await Promise.all([
          chatAgentsApi.get(member.agent_id),
          session.project_id ? projectApi.listMembers(session.project_id) : [],
        ]);
        const projectMember = projectMembers.find(item => item.id === member.project_member_id)
          ?? projectMembers.find(item => item.agent_id === agent.id);
        if (!active) return;
        setProfile({ agent, projectMember });
        setRole(agent.system_prompt);
        setModel(projectMember ? projectMember.execution_config.model_name ?? '' : agent.model_name ?? '');
        const runner = projectMember?.execution_config.runner_type ?? member.execution_config?.runner_type ?? agent.runner_type;
        void agentRuntimeApi.list().then(result => {
          if (active) setModels(result.runners.find(item => item.runner_type === runner)?.discovered_models ?? []);
        }).catch(() => undefined);
      } catch (err) {
        if (active) setError(err instanceof Error ? err.message : String(err));
      }
    })();
    return () => { active = false; };
  }, [activeSessionId, memberId, zh]);

  const save = async () => {
    if (!profile || saving) return;
    const { agent, projectMember } = profile;
    setError('');
    setSaving(true);
    try {
      const chosenModel = model.trim();
      if (projectMember && chosenModel !== (projectMember.execution_config.model_name ?? '')) {
        await projectApi.updateMember(projectMember.project_id, projectMember.id, {
          role: null,
          display_order: null,
          default_workspace_path: null,
          is_default: null,
          allowed_skill_ids: null,
          execution_config: { ...projectMember.execution_config, model_name: chosenModel || null },
        });
      }
      if (role !== agent.system_prompt || (!projectMember && chosenModel !== (agent.model_name ?? ''))) {
        await chatAgentsApi.update(agent.id, {
          name: null, runner_type: null, tools_enabled: null,
          system_prompt: role === agent.system_prompt ? null : role,
          model_name: projectMember ? null : chosenModel,
        });
      }
      await refreshMembers();
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="ui-sheet-backdrop" onClick={saving ? undefined : onClose}>
      <form className="ui-sheet ui-agent-profile" role="dialog" aria-modal="true"
        aria-label={zh ? '成员设置' : 'Member settings'}
        onClick={event => event.stopPropagation()}
        onKeyDown={event => { if (event.key === 'Escape' && !saving) onClose(); }}
        onSubmit={event => { event.preventDefault(); void save(); }}>
        <div className="ui-sheet-head">
          <strong>{profile ? profile.agent.name : zh ? '成员设置' : 'Member settings'}</strong>
          <button type="button" className="ui-sheet-close" disabled={saving} onClick={onClose}
            aria-label={zh ? '关闭' : 'Close'}><X size={18} /></button>
        </div>
        {profile && <>
          <label>{zh ? '模型' : 'Model'}
            <input list={listId} value={model} disabled={saving} onChange={event => setModel(event.target.value)}
              placeholder={zh ? '自动（沿用运行时默认模型）' : 'Automatic (runtime default)'} />
          </label>
          <datalist id={listId}>{models.map(name => <option key={name} value={name} />)}</datalist>
          <label>{zh ? '角色说明' : 'Role instructions'}
            <textarea value={role} disabled={saving} onChange={event => setRole(event.target.value)} rows={4} />
          </label>
          <div className="ui-agent-role-options">
            {(zh
              ? [['开发者', '实现任务并提供测试结果。'], ['审核员', '审查实际变更和测试，报告发现。'], ['研究员', '查证资料并提供来源。']]
              : [['Developer', 'Implement the task and report test results.'], ['Reviewer', 'Review actual changes and tests. Report findings.'], ['Researcher', 'Verify information and cite sources.']]
            ).map(([name, instructions]) => (
              <button key={name} type="button" disabled={saving} onClick={() => setRole(instructions)}>{name}</button>
            ))}
          </div>
          <p className="ui-sheet-hint">{zh
            ? `角色说明用于这个 Agent 的所有会话。模型设置用于${profile.projectMember ? '此项目' : '这个 Agent'}。`
            : `Role instructions apply to this agent across chats. The model setting applies to ${profile.projectMember ? 'this project' : 'this agent'}.`}</p>
          <button className="ui-sheet-primary" type="submit" disabled={saving}>{zh ? '保存' : 'Save'}</button>
        </>}
        {error && <p role="alert">{error}</p>}
        {!profile && !error && <p role="status">{zh ? '加载中…' : 'Loading…'}</p>}
      </form>
    </div>
  );
}
