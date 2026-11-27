import { useEffect, useState } from 'react';
import { useWorkspace } from '@/context/WorkspaceContext';
import { ActivityLogSheet } from './ActivityLogSheet';
import { SchedulesSheet } from './SchedulesSheet';

export type AppPanel = 'schedules' | 'activity' | 'brain';

const EVENT = 'onebrain:open-panel';

/** Open a group panel from anywhere (title bar, sidebar, shortcuts). */
export function openAppPanel(panel: AppPanel) {
  window.dispatchEvent(new CustomEvent<AppPanel>(EVENT, { detail: panel }));
}

/** Renders whichever group panel was asked for, for the current group. */
export function AppPanels() {
  const { activeSessionId } = useWorkspace();
  const [panel, setPanel] = useState<AppPanel | null>(null);

  useEffect(() => {
    // 'brain' is a page; the app shell opens it, this host only shows sheets.
    const open = (event: Event) => {
      const next = (event as CustomEvent<AppPanel>).detail;
      if (next !== 'brain') setPanel(next);
    };
    window.addEventListener(EVENT, open);
    return () => window.removeEventListener(EVENT, open);
  }, []);

  if (!panel || !activeSessionId) return null;
  const close = () => setPanel(null);
  return panel === 'schedules'
    ? <SchedulesSheet sessionId={activeSessionId} onClose={close} />
    : <ActivityLogSheet sessionId={activeSessionId} onClose={close} />;
}
