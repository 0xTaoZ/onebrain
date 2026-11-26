import { PanelRight, Users } from 'lucide-react';
import { useWorkspace } from '@/context/WorkspaceContext';
import { openGroupMemberPicker } from '@/lib/teamNavigation';

export function PaperChatHeader({ filesOpen, filesAvailable, onToggleFiles }: { filesOpen: boolean; filesAvailable: boolean; onToggleFiles: () => void }) {
  const { sessions, activeSessionId, members, membersAsync, locale } = useWorkspace();
  const zh = locale === 'zh';
  const session = sessions.find(item => item.id === activeSessionId);
  return <header className="paper-chat-heading">
    <div><h1>{session?.title ?? (zh ? '协作聊天' : 'Team chat')}</h1><p>{membersAsync.loading ? (zh ? '正在加载成员…' : 'Loading members…') : membersAsync.error ? (zh ? '成员信息不可用' : 'Members unavailable') : (zh ? `${members.length} 个 Agent · 你` : `${members.length} agents · you`)}</p></div>
    <div className="paper-chat-actions">
      <button type="button" className="paper-icon-button" aria-label={zh ? '添加成员' : 'Add members'} title={zh ? '添加成员' : 'Add members'} onClick={openGroupMemberPicker}><Users size={20} /></button>
      <button type="button" className="paper-icon-button" aria-label={zh ? '文件与变更' : 'Files and changes'} disabled={!filesAvailable} title={filesAvailable ? (zh ? '文件与变更' : 'Files and changes') : (zh ? '请放大窗口查看文件与变更' : 'Widen the window to view files and changes')} aria-pressed={filesOpen} onClick={onToggleFiles}><PanelRight size={20} /></button>
    </div>
  </header>;
}
