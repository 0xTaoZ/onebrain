import { Brain, CalendarClock, History, PanelRight, Users } from 'lucide-react';
import { useWorkspace } from '@/context/WorkspaceContext';
import { openGroupMemberPicker } from '@/lib/teamNavigation';
import { openAppPanel } from './AppPanels';

export function PaperChatHeader({ filesOpen, filesAvailable, onToggleFiles }: { filesOpen: boolean; filesAvailable: boolean; onToggleFiles: () => void }) {
  const { sessions, activeSessionId, members, membersAsync, locale } = useWorkspace();
  const zh = locale === 'zh';
  const session = sessions.find(item => item.id === activeSessionId);
  const working = members.filter(member => member.status === 'run').length;
  const subtitle = membersAsync.loading
    ? (zh ? '正在加载成员…' : 'Loading members…')
    : membersAsync.error
      ? (zh ? '成员信息不可用' : 'Members unavailable')
      : working > 0
        ? (zh ? `${members.length} 位成员 · ${working} 位正在工作` : `${members.length} members · ${working} working`)
        : (zh ? `${members.length} 位成员` : `${members.length} members`);
  const tool = (label: string, onClick: () => void, icon: React.ReactNode) => (
    <button type="button" className="paper-icon-button" aria-label={label} title={label} onClick={onClick}>{icon}</button>
  );
  return <header className="paper-chat-heading">
    <div><h1>{session?.title ?? (zh ? '协作聊天' : 'Team chat')}</h1><p>{subtitle}</p></div>
    <div className="paper-chat-actions">
      {tool(zh ? '共享大脑' : 'Shared brain', () => openAppPanel('brain'), <Brain size={20} />)}
      {tool(zh ? '定时任务' : 'Schedules', () => openAppPanel('schedules'), <CalendarClock size={20} />)}
      {tool(zh ? '活动记录' : 'Activity log', () => openAppPanel('activity'), <History size={20} />)}
      <span className="paper-chat-actions-divider" aria-hidden="true" />
      {tool(zh ? '添加成员' : 'Add members', openGroupMemberPicker, <Users size={20} />)}
      <button type="button" className="paper-icon-button" aria-label={zh ? '文件与变更' : 'Files and changes'} disabled={!filesAvailable} title={filesAvailable ? (zh ? '文件与变更' : 'Files and changes') : (zh ? '请放大窗口查看文件与变更' : 'Widen the window to view files and changes')} aria-pressed={filesOpen} onClick={onToggleFiles}><PanelRight size={20} /></button>
    </div>
  </header>;
}
