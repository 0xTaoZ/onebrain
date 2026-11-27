import { useCallback, useEffect, useMemo, useState } from 'react';
import { X } from 'lucide-react';
import { chatSchedulesApi } from '@/lib/api';
import { useWorkspace } from '@/context/WorkspaceContext';
import {
  ChatScheduleKind,
  ChatScheduleMisfire,
  type ChatSchedule,
  type ChatScheduleInput,
} from '../../../shared/types';

type Mode = 'daily' | 'weekdays' | 'weekly' | 'interval' | 'once' | 'cron';

type Draft = {
  title: string;
  prompt: string;
  mode: Mode;
  time: string;
  weekday: number;
  every: number;
  unit: 'minutes' | 'hours';
  onceAt: string;
  cron: string;
  skipMissed: boolean;
};

const EMPTY: Draft = {
  title: '',
  prompt: '',
  mode: 'daily',
  time: '09:00',
  weekday: 1,
  every: 30,
  unit: 'minutes',
  onceAt: '',
  cron: '0 9 * * 1-5',
  skipMissed: false,
};

function toInput(draft: Draft, enabled = true): ChatScheduleInput {
  const [hour, minute] = draft.time.split(':').map(part => Number(part) || 0);
  const base = {
    title: draft.title.trim(),
    prompt: draft.prompt.trim(),
    misfire: draft.skipMissed ? ChatScheduleMisfire.skip : ChatScheduleMisfire.run_once,
    enabled,
    cron_expr: null,
    every_seconds: null,
    run_at: null,
  };
  switch (draft.mode) {
    case 'daily':
      return { ...base, kind: ChatScheduleKind.cron, cron_expr: `${minute} ${hour} * * *` };
    case 'weekdays':
      return { ...base, kind: ChatScheduleKind.cron, cron_expr: `${minute} ${hour} * * 1-5` };
    case 'weekly':
      return { ...base, kind: ChatScheduleKind.cron, cron_expr: `${minute} ${hour} * * ${draft.weekday}` };
    case 'interval':
      return { ...base, kind: ChatScheduleKind.every, every_seconds: draft.every * (draft.unit === 'hours' ? 3600 : 60) };
    case 'once':
      return { ...base, kind: ChatScheduleKind.once, run_at: draft.onceAt ? new Date(draft.onceAt).toISOString() : null };
    case 'cron':
      return { ...base, kind: ChatScheduleKind.cron, cron_expr: draft.cron.trim() };
  }
}

function fromSchedule(schedule: ChatSchedule): ChatScheduleInput {
  return {
    title: schedule.title,
    prompt: schedule.prompt,
    kind: schedule.kind,
    cron_expr: schedule.cron_expr,
    every_seconds: schedule.every_seconds,
    run_at: schedule.run_at,
    misfire: schedule.misfire,
    enabled: schedule.enabled,
  };
}

/** Messages sent to this group on a schedule, so agents can work unattended. */
export function SchedulesSheet({ sessionId, onClose }: { sessionId: string; onClose: () => void }) {
  const { locale, showToast } = useWorkspace();
  const zh = locale === 'zh';
  const [schedules, setSchedules] = useState<ChatSchedule[] | null>(null);
  const [draft, setDraft] = useState<Draft | null>(null);
  const [preview, setPreview] = useState<{ times: string[]; error: string | null }>({ times: [], error: null });
  const [busy, setBusy] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

  const fmt = useMemo(
    () => new Intl.DateTimeFormat(locale, { weekday: 'short', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }),
    [locale],
  );
  const fail = useCallback((error: unknown) => showToast(error instanceof Error ? error.message : String(error)), [showToast]);
  const reload = useCallback(() => chatSchedulesApi.list(sessionId).then(setSchedules).catch(fail), [sessionId, fail]);
  useEffect(() => { void reload(); }, [reload]);

  // Show the next runs while the user edits, using the same rules as the backend.
  useEffect(() => {
    if (!draft) return;
    const input = toInput({ ...draft, title: draft.title || '-', prompt: draft.prompt || '-' });
    const timer = window.setTimeout(() => {
      chatSchedulesApi.preview(input)
        .then(result => setPreview({ times: result.times, error: null }))
        .catch(error => setPreview({ times: [], error: error instanceof Error ? error.message : String(error) }));
    }, 250);
    return () => window.clearTimeout(timer);
  }, [draft]);

  const save = async () => {
    if (!draft) return;
    setBusy(true);
    try {
      await chatSchedulesApi.create(sessionId, toInput(draft));
      setDraft(null);
      await reload();
      showToast(zh ? '定时任务已保存' : 'Schedule saved');
    } catch (error) {
      fail(error);
    } finally {
      setBusy(false);
    }
  };

  const toggle = async (schedule: ChatSchedule) => {
    try {
      await chatSchedulesApi.update(schedule.id, { ...fromSchedule(schedule), enabled: !schedule.enabled });
      await reload();
    } catch (error) {
      fail(error);
    }
  };

  const remove = async (schedule: ChatSchedule) => {
    if (confirmDelete !== schedule.id) {
      setConfirmDelete(schedule.id);
      return;
    }
    setConfirmDelete(null);
    try {
      await chatSchedulesApi.remove(schedule.id);
      await reload();
    } catch (error) {
      fail(error);
    }
  };

  const runNow = async (schedule: ChatSchedule) => {
    try {
      await chatSchedulesApi.runNow(schedule.id);
      showToast(zh ? '已发送到群里' : 'Sent to the group');
      window.setTimeout(() => void reload(), 1500);
    } catch (error) {
      fail(error);
    }
  };

  const describe = (schedule: ChatSchedule) => {
    if (schedule.kind === ChatScheduleKind.every && schedule.every_seconds) {
      const minutes = Number(schedule.every_seconds) / 60;
      return minutes % 60 === 0
        ? (zh ? `每 ${minutes / 60} 小时` : `Every ${minutes / 60} h`)
        : (zh ? `每 ${minutes} 分钟` : `Every ${minutes} min`);
    }
    if (schedule.kind === ChatScheduleKind.once) return zh ? '运行一次' : 'Once';
    return schedule.cron_expr ?? '';
  };
  const status = (schedule: ChatSchedule) => {
    switch (schedule.last_status) {
      case 'sent': return zh ? '上次：已发送' : 'Last: sent';
      case 'skipped_busy': return zh ? '上次：群里还在忙，跳过' : 'Last: skipped, group was busy';
      case 'skipped_missed': return zh ? '上次：电脑没开，跳过' : 'Last: skipped, app was closed';
      case 'failed': return zh ? '上次：失败' : 'Last: failed';
      default: return zh ? '还没运行过' : 'Not run yet';
    }
  };

  const set = (patch: Partial<Draft>) => setDraft(current => (current ? { ...current, ...patch } : current));
  const weekdays = zh ? ['周日', '周一', '周二', '周三', '周四', '周五', '周六'] : ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  const modes: Array<[Mode, string]> = [
    ['daily', zh ? '每天' : 'Daily'],
    ['weekdays', zh ? '工作日' : 'Weekdays'],
    ['weekly', zh ? '每周' : 'Weekly'],
    ['interval', zh ? '每隔' : 'Every'],
    ['once', zh ? '一次' : 'Once'],
    ['cron', 'cron'],
  ];

  return (
    <div className="ui-sheet-backdrop" role="dialog" aria-modal="true" aria-label={zh ? '定时任务' : 'Schedules'} onClick={onClose}>
      <div className="ui-sheet ui-sheet-wide" onClick={event => event.stopPropagation()}>
        <div className="ui-sheet-head">
          <strong>{zh ? '定时任务' : 'Schedules'}</strong>
          <button type="button" className="ui-sheet-close" onClick={onClose} aria-label={zh ? '关闭' : 'Close'}><X size={18} /></button>
        </div>
        <p className="ui-sheet-hint">
          {zh
            ? '到时间就把这条消息发到群里，用 @ 指定谁来做。审批照常需要你确认；App 开着时电脑不会自动睡眠。'
            : 'At the set time this message is posted to the group; use @ to pick who works on it. Approvals still ask you, and the Mac stays awake while the app is open.'}
        </p>

        {draft ? (
          <div className="ui-schedule-form">
            <input className="ui-input" placeholder={zh ? '名称，例如「每晚代码审查」' : 'Name, e.g. Nightly review'} value={draft.title} onChange={event => set({ title: event.target.value })} />
            <textarea className="ui-input" rows={3} placeholder={zh ? '要发的消息，例如「@codex 审查今天的改动」' : 'Message, e.g. @codex review today\'s changes'} value={draft.prompt} onChange={event => set({ prompt: event.target.value })} />
            <div className="ui-segment" role="radiogroup">
              {modes.map(([id, label]) => (
                <button key={id} type="button" role="radio" aria-checked={draft.mode === id} onClick={() => set({ mode: id })}>{label}</button>
              ))}
            </div>
            <div className="ui-schedule-when">
              {draft.mode === 'weekly' && (
                <select className="ui-input" value={draft.weekday} onChange={event => set({ weekday: Number(event.target.value) })}>
                  {weekdays.map((name, index) => <option key={name} value={index}>{name}</option>)}
                </select>
              )}
              {['daily', 'weekdays', 'weekly'].includes(draft.mode) && (
                <input className="ui-input" type="time" value={draft.time} onChange={event => set({ time: event.target.value })} />
              )}
              {draft.mode === 'interval' && (
                <>
                  <input className="ui-input" type="number" min={1} value={draft.every} onChange={event => set({ every: Math.max(1, Number(event.target.value) || 1) })} />
                  <select className="ui-input" value={draft.unit} onChange={event => set({ unit: event.target.value as Draft['unit'] })}>
                    <option value="minutes">{zh ? '分钟' : 'minutes'}</option>
                    <option value="hours">{zh ? '小时' : 'hours'}</option>
                  </select>
                </>
              )}
              {draft.mode === 'once' && (
                <input className="ui-input" type="datetime-local" value={draft.onceAt} onChange={event => set({ onceAt: event.target.value })} />
              )}
              {draft.mode === 'cron' && (
                <input className="ui-input ui-mono" value={draft.cron} onChange={event => set({ cron: event.target.value })} placeholder="0 9 * * 1-5" />
              )}
            </div>
            <label className="ui-audit-filter">
              <input type="checkbox" checked={draft.skipMissed} onChange={event => set({ skipMissed: event.target.checked })} />
              {zh ? '电脑没开时错过的就跳过（默认补跑一次）' : 'Skip runs missed while the app was closed (default: run once)'}
            </label>
            <p className="ui-sheet-hint">
              {preview.error
                ? preview.error
                : preview.times.length
                  ? `${zh ? '接下来：' : 'Next: '}${preview.times.slice(0, 3).map(time => fmt.format(new Date(time))).join(' · ')}`
                  : (zh ? '这个时间不会再运行' : 'This never runs again')}
            </p>
            <div className="ui-schedule-actions">
              <button type="button" className="ui-sheet-done" onClick={() => setDraft(null)}>{zh ? '取消' : 'Cancel'}</button>
              <button type="button" className="ui-sheet-primary" disabled={busy || !draft.title.trim() || !draft.prompt.trim() || preview.error !== null} onClick={() => void save()}>
                {busy ? '…' : (zh ? '保存' : 'Save')}
              </button>
            </div>
          </div>
        ) : (
          <button type="button" className="ui-sheet-primary" onClick={() => setDraft(EMPTY)}>{zh ? '新建定时任务' : 'New schedule'}</button>
        )}

        <ul className="ui-sheet-list">
          {(schedules ?? []).map(schedule => (
            <li key={schedule.id} data-disabled={!schedule.enabled || undefined}>
              <span className="ui-sheet-name">
                <strong>{schedule.title}</strong>
                <small>
                  {describe(schedule)}
                  {schedule.enabled && schedule.next_run_at ? ` · ${zh ? '下次' : 'next'} ${fmt.format(new Date(schedule.next_run_at))}` : ` · ${zh ? '已暂停' : 'paused'}`}
                  {` · ${status(schedule)}`}
                </small>
              </span>
              <button type="button" className="ui-sheet-done" onClick={() => void runNow(schedule)}>{zh ? '立即运行' : 'Run now'}</button>
              <button type="button" className="ui-sheet-done" onClick={() => void toggle(schedule)}>{schedule.enabled ? (zh ? '暂停' : 'Pause') : (zh ? '启用' : 'Resume')}</button>
              <button type="button" className="ui-sheet-done" data-danger={confirmDelete === schedule.id || undefined} onClick={() => void remove(schedule)}>
                {confirmDelete === schedule.id ? (zh ? '确认删除' : 'Confirm') : (zh ? '删除' : 'Delete')}
              </button>
            </li>
          ))}
        </ul>
        {schedules?.length === 0 && !draft && <p className="ui-sheet-hint">{zh ? '还没有定时任务。' : 'No schedules yet.'}</p>}
      </div>
    </div>
  );
}
