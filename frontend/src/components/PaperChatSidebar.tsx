import { AgentProfileSheet } from './AgentProfileSheet';
import { useId, useState, type ReactNode } from 'react';
import { ArrowLeft, FolderKanban, MessageCircle, Plus, Search, Settings, Users, X } from 'lucide-react';
import { useWorkspace } from '@/context/WorkspaceContext';
import { AgentCharacterAvatar } from './AgentCharacterAvatar';
import { MemberActivityHeatmap } from './MemberActivityHeatmap';
import { openAppPanel } from './AppPanels';
import { QuotaBars, useRuntimeQuota } from './QuotaBars';
import { openGroupMemberPicker } from '@/lib/teamNavigation';
import { copyPlainText } from '@/lib/clipboard';
import { projectDisplayName } from '@/lib/projectDisplay';
import type { Member, SidebarNavigationTarget } from '@/types';

export function PaperMemberAvatar({ member }: { member: Member }) {
  return <span className="paper-person-avatar"><AgentCharacterAvatar identity={`${member.runner ?? ''} ${member.name} ${member.modelName}`}>{member.avatar}</AgentCharacterAvatar></span>;
}

export function PaperChatSidebar({ activePage, workspaceNavigation, onSessionSelect, onCreateSession, onNavigate }: {
  activePage: SidebarNavigationTarget;
  workspaceNavigation: ReactNode;
  onSessionSelect: (id: string) => void;
  onCreateSession: () => void;
  onNavigate: (page: SidebarNavigationTarget) => void;
}) {
  const { locale, members, membersAsync, sessions, sessionsAsync, activeSessionId, messages, messagesAsync, projects, selectedProjectId, setSelectedProjectId, refreshSessions, showToast } = useWorkspace();
  const zh = locale === 'zh';
  const panelId = useId();
  const [tab, setTab] = useState<'chats' | 'contacts'>('chats');
  const quotas = useRuntimeQuota();
  // A member without a chosen model shows that it uses its CLI's default.
  const modelLabel = (member: Member) =>
    !member.modelName || member.modelName.toUpperCase() === member.modelName && member.modelName === member.runner
      ? (zh ? '默认模型' : 'Default model')
      : member.modelName;
  const [selectedMemberId, setSelectedMemberId] = useState<string | null>(null);

  const [showWorkspace, setShowWorkspace] = useState(false);
  const [editingMemberId, setEditingMemberId] = useState<string | null>(null);
  const [query, setQuery] = useState('');
  const visibleMembers = membersAsync.loading || membersAsync.error ? [] : members;
  const selectedMember = visibleMembers.find(member => member.id === selectedMemberId);
  // Pages such as settings or the shared brain keep the chat list beside them.
  const navigationVisible = showWorkspace;
  const status = (member: Member) => member.status === 'run' ? (zh ? '正在工作' : 'Working') : member.status === 'on' ? (zh ? '空闲' : 'Idle') : (zh ? '不可用' : 'Unavailable');
  const showChats = () => { setShowWorkspace(false); setSelectedMemberId(null); setTab('chats'); onNavigate('workspace'); };
  const showMember = (id: string) => { setSelectedMemberId(id); setShowWorkspace(false); setTab('contacts'); onNavigate('workspace'); };
  const latestMessage = !messagesAsync.loading && !messagesAsync.error ? messages.filter(message => !message.isThinking).at(-1) : undefined;

  return (
    <div className="paper-navigation">
      {editingMemberId && <AgentProfileSheet key={editingMemberId} memberId={editingMemberId} onClose={() => setEditingMemberId(null)} />}
      <nav className="paper-agent-rail" aria-label={zh ? '成员与工作台' : 'Members and workspace'}>
        <button type="button" className="paper-rail-person" aria-pressed={!navigationVisible && !selectedMember} onClick={showChats}>
          <span className="paper-person-avatar paper-self-avatar">{zh ? '我' : 'Me'}</span>
          <span>{zh ? '我' : 'Me'}</span>
        </button>
        <span className="paper-rail-divider" />
        <div className="paper-rail-members">
          {visibleMembers.map(member => (
            <button type="button" key={member.id} className="paper-rail-person" data-status={member.status} aria-pressed={!navigationVisible && selectedMember?.id === member.id} onClick={() => showMember(member.id)} title={`${member.name} · ${status(member)}`}>
              <PaperMemberAvatar member={member} />
              <span className="paper-status" data-status={member.status}><i />{status(member)}</span>
              <span className="paper-rail-name">{member.name.replace(/^@/, '')}</span>
            </button>
          ))}
          {membersAsync.loading && <span className="paper-rail-hint">{zh ? '加载中' : 'Loading'}</span>}
        </div>
        <button type="button" className="paper-rail-person paper-connect" onClick={openGroupMemberPicker}>
          <span className="paper-connect-avatar"><Plus size={24} /></span><span>{zh ? '接入成员' : 'Add agent'}</span>
        </button>
        <div className="paper-rail-footer">
          <button type="button" className="paper-rail-person" aria-pressed={navigationVisible} onClick={() => setShowWorkspace(value => !value)}>
            <span className="paper-rail-tool"><FolderKanban size={21} /></span><span>{zh ? '工作台' : 'Workspace'}</span>
          </button>
          <button type="button" className="paper-rail-person" onClick={() => onNavigate('providers')}>
            <span className="paper-rail-tool"><Settings size={21} /></span><span>{zh ? '设置' : 'Settings'}</span>
          </button>
        </div>
      </nav>

      <section className="paper-conversation-panel" aria-label={zh ? '会话与通讯录' : 'Conversations and contacts'}>
        {navigationVisible ? <>
          <button type="button" className="paper-back" onClick={showChats}><ArrowLeft size={17} />{zh ? '返回聊天' : 'Back to chats'}</button>
          <div className="paper-original-navigation">{workspaceNavigation}</div>
        </> : <>
          <div className="paper-profile">
            {selectedMember ? <PaperMemberAvatar member={selectedMember} /> : <img className="paper-person-avatar" src="/logos/onebrain-logo.png" alt="" />}
            <div><strong>{selectedMember?.name.replace(/^@/, '') ?? 'OneBrain'}</strong><span>{selectedMember ? (zh ? '当前群成员资料' : 'Current group member') : (zh ? '你的协作空间' : 'Your team space')}</span></div>
            <button type="button" className="paper-icon-button" aria-label={selectedMember ? (zh ? '成员设置' : 'Member settings') : (zh ? '添加成员' : 'Add members')} onClick={() => selectedMember ? setEditingMemberId(selectedMember.id) : openGroupMemberPicker()}><Settings size={19} /></button>
          </div>
          {projects.length > 1 && <label className="paper-project-picker"><FolderKanban size={15} /><select aria-label={zh ? '选择项目' : 'Choose project'} value={selectedProjectId} onChange={event => { setSelectedMemberId(null); setSelectedProjectId(event.target.value); }}>
            {projects.map(project => <option key={project.id} value={project.id}>{projectDisplayName(project)}</option>)}
          </select></label>}
          <div className="paper-conversation-tabs" role="tablist" aria-label={zh ? '聊天与通讯录' : 'Chats and contacts'}>
            <button type="button" role="tab" aria-selected={tab === 'chats'} aria-controls={`${panelId}-paper-chats-panel`} id={`${panelId}-paper-chats-tab`} onClick={() => { setTab('chats'); setSelectedMemberId(null); }}><MessageCircle size={17} />{zh ? '聊天' : 'Chats'}</button>
            <button type="button" role="tab" aria-selected={tab === 'contacts'} aria-controls={`${panelId}-paper-contacts-panel`} id={`${panelId}-paper-contacts-tab`} onClick={() => setTab('contacts')}><Users size={17} />{zh ? '通讯录' : 'Contacts'}</button>
          </div>
          <button type="button" className="paper-create-group" onClick={tab === 'chats' ? onCreateSession : openGroupMemberPicker}><Plus size={20} />{tab === 'chats' ? (zh ? '建群' : 'New group') : (zh ? '添加成员' : 'Add member')}</button>
          <label className="paper-conversation-search"><Search size={16} /><input aria-label={zh ? '搜索会话或成员' : 'Search conversations or members'} placeholder={zh ? '搜索' : 'Search'} value={query} onChange={event => setQuery(event.target.value)} />{query && <button type="button" aria-label={zh ? '清除搜索' : 'Clear search'} onClick={() => setQuery('')}><X size={14} /></button>}</label>
          <div className="paper-conversation-scroll" role="tabpanel" id={`${panelId}-${tab === 'chats' ? 'paper-chats-panel' : 'paper-contacts-panel'}`} aria-labelledby={`${panelId}-${tab === 'chats' ? 'paper-chats-tab' : 'paper-contacts-tab'}`}>
            {tab === 'chats' ? <>
              {sessionsAsync.error && <div className="paper-list-notice" role="alert">{zh ? '会话加载失败' : 'Could not load conversations'}<button type="button" onClick={() => void refreshSessions()}>{zh ? '重试' : 'Retry'}</button></div>}
              {sessionsAsync.loading && <p className="paper-list-notice" role="status">{zh ? '正在加载会话…' : 'Loading conversations…'}</p>}
              {sessions.filter(session => session.title.toLocaleLowerCase().includes(query.toLocaleLowerCase())).map(session => {
                const active = session.id === activeSessionId;
                const pending = session.hasPendingApproval || session.hasPendingWorkflowInput || session.hasPendingWorkflowReview;
                const preview = active && latestMessage ? `${latestMessage.isUser ? (zh ? '你' : 'You') : latestMessage.sender.replace(/^@/, '')}: ${latestMessage.text}` : session.hasRunningAgent || session.hasRunningWorkflow ? (zh ? '成员正在工作' : 'Agents are working') : pending ? (zh ? '需要你处理' : 'Needs your attention') : (zh ? '打开会话，继续协作' : 'Open conversation');
                return <button type="button" key={session.id} className="paper-conversation-row" aria-current={active ? 'page' : undefined} onClick={() => onSessionSelect(session.id)}>
                  <span className={`paper-group-avatar ${active && visibleMembers.length ? 'paper-group-avatar-grid' : ''}`}>
                    {active && visibleMembers.length ? visibleMembers.slice(0, 3).map(member => <PaperMemberAvatar key={member.id} member={member} />) : <Users size={25} />}
                    {active && visibleMembers.length > 0 && <span className="paper-mini-self">{zh ? '我' : 'Me'}</span>}
                  </span>
                  <span className="paper-conversation-copy"><span className="paper-conversation-title">{session.title}</span><span className="paper-conversation-preview">{preview}</span></span>
                  {(session.hasUnreadAgentCompletion || pending) && <span className="paper-unread-dot" aria-label={zh ? '有待处理消息' : 'Unread activity'} />}
                </button>;
              })}
              {!sessionsAsync.loading && !sessionsAsync.error && sessions.length === 0 && <p className="paper-list-notice">{zh ? '还没有群聊，点击「建群」开始。' : 'Create your first group to start.'}</p>}
            </> : <>
              <p className="paper-contact-caption">{zh ? '当前群聊成员' : 'Members of this group'}</p>
              {membersAsync.error && <p className="paper-list-notice" role="alert">{zh ? '成员加载失败，请到工作台重试。' : 'Could not load members. Retry in the workspace.'}</p>}
              {membersAsync.loading && <p className="paper-list-notice" role="status">{zh ? '正在加载成员…' : 'Loading members…'}</p>}
              {visibleMembers.filter(member => member.name.toLocaleLowerCase().includes(query.toLocaleLowerCase())).map(member => <div key={member.id} className="paper-contact">
                <button type="button" className="paper-contact-heading" aria-expanded={selectedMember?.id === member.id} onClick={() => setSelectedMemberId(selectedMember?.id === member.id ? null : member.id)}>
                  <PaperMemberAvatar member={member} /><span><strong>{member.name.replace(/^@/, '')}</strong><small>{modelLabel(member)}</small>{(() => { const quota = quotas.find(item => item.runner === member.runner); return quota ? <QuotaBars quota={quota} zh={zh} /> : null; })()}</span><span className="paper-contact-expand">{selectedMember?.id === member.id ? '−' : '+'}</span>
                </button>
                {selectedMember?.id === member.id && <div className="paper-contact-detail"><span className="paper-status" data-status={member.status}><i />{status(member)}</span><p>{member.roleDetail}</p><button type="button" onClick={() => setEditingMemberId(member.id)}>{zh ? `换模型（现在：${modelLabel(member)}）` : `Change model (now: ${modelLabel(member)})`}</button><MemberActivityHeatmap sessionAgentId={member.id} label={total => (zh ? `近半年运行 ${total} 次` : `${total} runs in the last 6 months`)} /><button type="button" onClick={() => { void copyPlainText(`@${member.name.replace(/^@/, '')} `).then(ok => showToast(ok ? (zh ? '已复制成员称呼，粘贴到聊天输入框即可。' : 'Mention copied. Paste it into the composer.') : (zh ? '复制失败，请在输入框中输入 @ 选择成员。' : 'Could not copy. Type @ in the composer.'))); }}>{zh ? '复制 @称呼' : 'Copy @mention'}</button></div>}
              </div>)}
              {!membersAsync.loading && !membersAsync.error && !visibleMembers.length && <p className="paper-list-notice">{zh ? '群聊还没有成员，添加后即可分派任务。' : 'Add a member to this group to assign tasks.'}</p>}
              {activeSessionId && <button type="button" className="paper-manage-members" onClick={() => onNavigate('brain')}>{zh ? '共享大脑' : 'Shared brain'}</button>}
              {activeSessionId && <button type="button" className="paper-manage-members" onClick={() => openAppPanel('schedules')}>{zh ? '定时任务' : 'Schedules'}</button>}
              {activeSessionId && <button type="button" className="paper-manage-members" onClick={() => openAppPanel('activity')}>{zh ? '活动记录' : 'Activity log'}</button>}
            </>}
          </div>
        </>}
      </section>
    </div>
  );
}
