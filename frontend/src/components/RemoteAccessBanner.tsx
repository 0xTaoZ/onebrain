import { useState } from 'react';
import { useWorkspace } from '@/context/WorkspaceContext';
import { isLocalDesktopOrigin } from '@/lib/lanApi';

export function RemoteAccessBanner() {
  const { locale } = useWorkspace();
  const [busy, setBusy] = useState(false);
  if (isLocalDesktopOrigin() || window.location.protocol !== 'https:') return null;
  const zh = locale === 'zh';
  const logout = async () => {
    setBusy(true);
    try {
      const response = await fetch('/lan/logout', { method: 'POST', credentials: 'same-origin' });
      if (!response.ok) throw new Error('logout failed');
    } finally { window.location.reload(); }
  };
  return <div className="flex shrink-0 items-center justify-between gap-2 bg-[var(--surface-2)] px-3 py-2 text-xs text-[var(--ink-subtle)]"><span>{zh ? '内网远程会话：正在控制运行 OpenTeams 的电脑' : 'LAN session: controlling the computer running OpenTeams'}</span><button type="button" disabled={busy} onClick={() => void logout()} className="shrink-0 text-[var(--primary)]">{zh ? '退出' : 'Sign out'}</button></div>;
}
