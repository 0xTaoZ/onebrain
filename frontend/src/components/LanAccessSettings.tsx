import { useEffect, useState } from 'react';
import { useWorkspace } from '@/context/WorkspaceContext';
import { isLocalDesktopOrigin, lanApi } from '@/lib/lanApi';
import { copyPlainText } from '@/lib/clipboard';
import type { LanStatus } from '../../../shared/types';

export function LanAccessSettings() {
  const { locale, showToast } = useWorkspace();
  const zh = locale === 'zh';
  const [status, setStatus] = useState<LanStatus | null>(null);
  const [address, setAddress] = useState('');
  const [secret, setSecret] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const local = isLocalDesktopOrigin();
  useEffect(() => {
    if (!local) return;
    let active = true;
    void lanApi.status().then(value => {
      if (!active) return;
      setStatus(value);
      if (value.suggested_bind_address) setAddress(value.suggested_bind_address);
    }).catch(() => { if (active) setError(zh ? '无法读取内网状态，请确认后台已更新。' : 'Cannot read LAN status. Check that the backend is up to date.'); });
    return () => { active = false; };
  }, [local, zh]);
  if (!local) return null;
  const act = async (action: 'enable' | 'disable' | 'rotate') => {
    if (busy) return;
    setBusy(true); setError(null);
    try {
      if (action === 'disable') { setStatus(await lanApi.disable()); setSecret(null); }
      else {
        const result = action === 'enable' ? await lanApi.enable({ bind_address: address.trim() }) : await lanApi.rotate();
        setStatus(result.status); setSecret(result.pairing_secret);
      }
    } catch {
      setError(zh ? '操作失败。请检查这台电脑的内网 IPv4 地址，确认端口可用后重试。' : 'Could not apply the change. Check this computer’s private IPv4 address and retry.');
    } finally { setBusy(false); }
  };
  const copy = async (text: string) => showToast(await copyPlainText(text) ? (zh ? '已复制' : 'Copied') : (zh ? '复制失败，请手动复制' : 'Could not copy; select the text manually'));
  return <section className="space-y-2.5" aria-labelledby="lan-settings-title">
    <h4 id="lan-settings-title" className="settings-section-header">{zh ? '手机与 iPad 内网访问' : 'Phone and iPad access over LAN'}</h4>
    <div className="settings-list-group rounded-lg border p-4 space-y-3">
      <p className="settings-row-description">{zh ? '在同一内网通过浏览器控制这台电脑上的 OneBrain。默认关闭，使用 HTTPS 和配对码。' : 'Control OneBrain on this computer from a browser on the same LAN. Off by default; uses HTTPS and a pairing code.'}</p>
      {!status?.enabled ? <div className="flex flex-wrap items-center gap-2">
        <input aria-label={zh ? '这台电脑的内网 IPv4 地址' : 'This computer’s private IPv4 address'} value={address} placeholder="192.168.1.10" onChange={event => setAddress(event.target.value)} className="rounded-md border border-[var(--hairline)] bg-[var(--surface-1)] px-3 py-2 text-sm" />
        <button type="button" disabled={!status || !address.trim() || busy} onClick={() => void act('enable')} className="rounded-md bg-[var(--primary)] px-3 py-2 text-sm text-white disabled:opacity-50">{zh ? '开启内网访问' : 'Enable LAN access'}</button>
      </div> : <>
        <div className="flex flex-wrap items-center gap-2"><a href={status.url ?? undefined} target="_blank" rel="noreferrer" className="break-all text-sm text-[var(--primary)]">{status.url}</a><button type="button" onClick={() => status.url && void copy(status.url)} className="text-xs text-[var(--primary)]">{zh ? '复制网址' : 'Copy URL'}</button></div>
        {secret && <div className="rounded-md bg-[var(--surface-2)] p-3 space-y-2"><p className="text-xs">{zh ? '在手机登录页输入配对码；此码仅可使用一次。退出后需在这台电脑上重新配对。' : 'Enter this single-use code on the phone’s sign-in page. After signing out, pair again from this computer.'}</p><code className="block break-all select-text text-sm">{secret}</code><button type="button" onClick={() => void copy(secret)} className="text-xs text-[var(--primary)]">{zh ? '复制配对码' : 'Copy pairing code'}</button></div>}
        <p className="settings-row-description">{zh ? '首次连接需在设备上确认本机自签名证书，并核对下方指纹。远程控制的文件和命令都在这台电脑上执行。' : 'On first connection, confirm this computer’s self-signed certificate and compare the fingerprint below. Remote files and commands run on this computer.'}</p>
        <p className="text-xs break-all select-text text-[var(--ink-tertiary)]">SHA-256: {status.certificate_sha256}</p>
        <div className="flex flex-wrap gap-3"><button type="button" disabled={busy} onClick={() => void act('rotate')} className="text-sm text-[var(--primary)]">{zh ? '重新配对并退出所有设备' : 'Pair again and sign out devices'}</button><button type="button" disabled={busy} onClick={() => void act('disable')} className="text-sm text-red-600">{zh ? '关闭内网访问' : 'Disable LAN access'}</button></div>
      </>}
      {error && <p role="alert" className="text-xs text-red-600">{error}</p>}
    </div>
  </section>;
}
