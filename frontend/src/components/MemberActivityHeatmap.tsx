import { useEffect, useMemo, useState } from 'react';
import type { DailyRunCount } from '../../../shared/types';
import { sessionAgentsApi } from '@/lib/api';

const WEEKS = 26;
const DAY_MS = 86_400_000;

function utcDay(date: Date): string {
  return date.toISOString().slice(0, 10);
}

/** Map a count to one of five shades, relative to the busiest day. */
function level(runs: number, busiest: number): number {
  if (runs <= 0 || busiest <= 0) return 0;
  return Math.min(4, Math.ceil((runs / busiest) * 4));
}

/** GitHub-style grid of daily runs for one member: columns are weeks, rows are weekdays. */
export function MemberActivityHeatmap({ sessionAgentId, label }: { sessionAgentId: string; label: (total: number) => string }) {
  const [days, setDays] = useState<DailyRunCount[] | null>(null);

  useEffect(() => {
    let cancelled = false;
    setDays(null);
    sessionAgentsApi.activity(sessionAgentId)
      .then(result => { if (!cancelled) setDays(result); })
      .catch(() => { if (!cancelled) setDays([]); });
    return () => { cancelled = true; };
  }, [sessionAgentId]);

  const { cells, total } = useMemo(() => {
    const counts = new Map((days ?? []).map(entry => [entry.day, entry.runs]));
    const today = new Date();
    const end = Date.UTC(today.getUTCFullYear(), today.getUTCMonth(), today.getUTCDate());
    const start = end - ((WEEKS - 1) * 7 + new Date(end).getUTCDay()) * DAY_MS;
    const list: Array<{ day: string; runs: number }> = [];
    for (let time = start; time <= end; time += DAY_MS) {
      const day = utcDay(new Date(time));
      list.push({ day, runs: counts.get(day) ?? 0 });
    }
    return { cells: list, total: list.reduce((sum, cell) => sum + cell.runs, 0) };
  }, [days]);

  if (days === null) return <div className="ui-heatmap ui-heatmap-loading" aria-hidden="true" />;
  const busiest = Math.max(0, ...cells.map(cell => cell.runs));

  return (
    <figure className="ui-heatmap" aria-label={label(total)}>
      <div className="ui-heatmap-grid">
        {cells.map(cell => (
          <span key={cell.day} className="ui-heatmap-cell" data-level={level(cell.runs, busiest)} title={`${cell.day} · ${cell.runs}`} />
        ))}
      </div>
      <figcaption>{label(total)}</figcaption>
    </figure>
  );
}
