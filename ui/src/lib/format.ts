import type { MediaInfo, Preset } from './api';
import { locale, t } from './i18n/index.svelte';

export function duration(secs: number | null | undefined): string {
  if (secs == null || !isFinite(secs)) return '';
  const total = Math.round(secs);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = String(total % 60).padStart(2, '0');
  return h > 0 ? `${h}:${String(m).padStart(2, '0')}:${s}` : `${m}:${s}`;
}

export function sampleRate(hz: number | null | undefined): string {
  if (!hz) return '';
  return t().units.khz((hz / 1000).toLocaleString(locale(), { maximumFractionDigits: 1 }));
}

export function extension(fileName: string): string {
  const dot = fileName.lastIndexOf('.');
  return dot > 0 ? fileName.slice(dot + 1).toLowerCase() : '';
}

/** Kurzbeschreibung der Quelle, z. B. ["FLAC", "96 kHz", "2.304 kbit/s", "24 Bit", "3:35"] */
export function describe(info: MediaInfo): string[] {
  const parts = [info.codec.toUpperCase(), sampleRate(info.sampleRate), bitRate(info.bitRate)];
  if (info.bitsPerSample) parts.push(t().units.bits(info.bitsPerSample));
  parts.push(duration(info.durationSecs));
  return parts.filter(Boolean);
}

export function bitRate(bps: number | null | undefined): string {
  if (!bps) return '';
  return t().units.kbps(Math.round(bps / 1000).toLocaleString(locale()));
}

/** Letzte zwei Pfadteile, damit lange Pfade in die Leiste passen. */
export function shortPath(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.length <= 2 ? path : `…/${parts.slice(-2).join('/')}`;
}

/** Kurzbeschreibung eines Presets, z. B. "VBR V0, max. 44,1 kHz, Mono" */
export function presetSummary(p: Preset): string {
  const m = t().presets;
  const parts: string[] = [];
  if (p.rate.mode === 'quality') parts.push(p.codec === 'mp3' ? m.vbr(p.rate.value) : m.quality(p.rate.value));
  else if (p.rate.mode === 'bitrate') parts.push(t().units.kbps(String(p.rate.kbps)));
  else parts.push(m.lossless);
  if (p.sampleRate) parts.push(m.atMost(sampleRate(p.sampleRate)));
  if (p.bitDepth) parts.push(m.atMost(t().units.bits(p.bitDepth)));
  if (p.channels) parts.push(p.channels === 1 ? m.mono : m.stereo);
  if (!p.sampleRate && !p.bitDepth && p.rate.mode === 'lossless') parts.push(m.asSource);
  return parts.join(', ');
}

/** Datum für Anzeigen, z. B. im Import-Dialog */
export function date(iso: string): string {
  const d = new Date(iso);
  return isNaN(d.getTime()) ? iso : d.toLocaleString(locale(), { dateStyle: 'medium', timeStyle: 'short' });
}
