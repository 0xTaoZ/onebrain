import { useEffect, useState } from 'react';
import { quotaApi } from '@/lib/api';
import type { RuntimeQuota } from '../../../shared/types';

let cache: { at: number; data: RuntimeQuota[] } | null = null;

/** Quota per runtime, refreshed every minute and shared by every caller. */
export function useRuntimeQuota(): RuntimeQuota[] {
  const [data, setData] = useState<RuntimeQuota[]>(cache?.data ?? []);
  useEffect(() => {
    let alive = true;
    const load = () => {
      if (cache && Date.now() - cache.at < 30_000) {
        setData(cache.data);
        return;
      }
      quotaApi.list()
        .then(result => {
          cache = { at: Date.now(), data: result };
          if (alive) setData(result);
        })
        .catch(() => undefined);
    };
    load();
    const timer = window.setInterval(load, 60_000);
    return () => { alive = false; window.clearInterval(timer); };
  }, []);
  return data;
}

/** Short and weekly usage of one runtime, as two thin bars. */
export function QuotaBars({ quota, zh }: { quota: RuntimeQuota; zh: boolean }) {
  const now = Date.now() / 1000;
  const rows = [
    { window: quota.primary, label: zh ? '5 小时' : '5 h' },
    { window: quota.secondary, label: zh ? '本周' : 'Week' },
  ].filter(row => row.window);
  if (rows.length === 0) return null;
  const resetText = (at: number | null) => {
    if (!at) return '';
    const minutes = Math.max(0, Math.round((at - now) / 60));
    if (minutes < 60) return zh ? `${minutes} 分钟后重置` : `resets in ${minutes} min`;
    const hours = Math.round(minutes / 60);
    if (hours < 48) return zh ? `${hours} 小时后重置` : `resets in ${hours} h`;
    return zh ? `${Math.round(hours / 24)} 天后重置` : `resets in ${Math.round(hours / 24)} d`;
  };
  return (
    <span className="ui-quota" title={quota.observed_at ? `${zh ? '记录于' : 'recorded'} ${new Date(quota.observed_at).toLocaleString()}` : undefined}>
      {rows.map(({ window, label }) => {
        // A window whose reset time has passed starts from zero again.
        const used = window!.resets_at && window!.resets_at < now ? 0 : window!.used_percent;
        return (
          <span key={label} className="ui-quota-row" data-level={used >= 90 ? 'high' : used >= 70 ? 'mid' : undefined}>
            <span className="ui-quota-label">{label}</span>
            <span className="ui-quota-track"><span style={{ width: `${Math.min(100, used)}%` }} /></span>
            <span className="ui-quota-value">{Math.round(used)}%</span>
            <span className="ui-quota-reset">{resetText(window!.resets_at)}</span>
          </span>
        );
      })}
    </span>
  );
}
