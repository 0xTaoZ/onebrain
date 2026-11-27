import { ArrowLeft, Menu } from 'lucide-react';
import type { ReactNode } from 'react';
import { useWorkspace } from '@/context/WorkspaceContext';

/**
 * The bar above the main view, in the style of a desktop chat app: one view at
 * a time, a back button on detail pages, and the group's people and tools on
 * the right. Replaces the browser-like tab strip.
 */
export function AppTitleBar({
  title,
  isSession,
  onBack,
  onOpenBrain,
  onOpenNavigation,
  trailing,
}: {
  title: string;
  isSession: boolean;
  onBack: () => void;
  onOpenBrain: () => void;
  onOpenNavigation: () => void;
  trailing?: ReactNode;
}) {
  const { locale, members } = useWorkspace();
  const zh = locale === 'zh';
  const working = members.filter(member => member.status === 'run').length;
  const subtitle = isSession
    ? working > 0
      ? (zh ? `${members.length} 位成员 · ${working} 位正在工作` : `${members.length} members · ${working} working`)
      : (zh ? `${members.length} 位成员` : `${members.length} members`)
    : null;

  if (isSession) {
    // In a chat the chat's own header is the title; keep only the phone menu.
    return (
      <div className="md:hidden">
        <div className="ui-titlebar ui-titlebar-compact" data-tauri-drag-region>
          <button type="button" className="ui-icon-btn" onClick={onOpenNavigation} aria-label={zh ? '打开导航' : 'Open navigation'}>
            <Menu size={16} />
          </button>
          {trailing}
        </div>
      </div>
    );
  }

  return (
    <header className="ui-titlebar" data-tauri-drag-region>
      <span className="md:hidden">
        <button type="button" className="ui-icon-btn" onClick={onOpenNavigation} aria-label={zh ? '打开导航' : 'Open navigation'}>
          <Menu size={16} />
        </button>
      </span>
      <button type="button" className="ui-back-btn" onClick={onBack}>
        <ArrowLeft size={16} />
        <span>{zh ? '返回聊天' : 'Back to chat'}</span>
      </button>
      <div className="ui-titlebar-text" data-tauri-drag-region>
        <strong>{title}</strong>
        {subtitle && <small>{subtitle}</small>}
      </div>
      <div className="ui-titlebar-actions">
        {trailing}
      </div>
    </header>
  );
}
