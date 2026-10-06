// Einstellungen, die lokal im WebView gespeichert werden.
import type { Codec } from './api';

export type Theme = 'system' | 'light' | 'dark';
export type Density = 'comfortable' | 'compact';
export type SortKey = 'added' | 'name' | 'sampleRate' | 'bitsPerSample' | 'bitRate';
export type SortDir = 'asc' | 'desc';
export type Language = 'system' | 'de' | 'en';
export type Mode = 'simple' | 'advanced';

export interface Settings {
  /** Sprache der Oberfläche; `system` folgt der Systemsprache */
  language: Language;
  /** Erweitert zeigt zusätzlich „Überschreiben“ und ffmpeg-Details. */
  mode: Mode;
  theme: Theme;
  density: Density;
  accent: string;
  /** Gewähltes Zielformat */
  format: Codec;
  /** Zuletzt gewähltes Preset je Format */
  presetByFormat: Partial<Record<Codec, string>>;
  outputDir: string | null;
  template: string;
  overwrite: boolean;
  subfolders: boolean;
  /** Sortierung der Dateiliste */
  sortKey: SortKey;
  sortDir: SortDir;
  /** Beim Start nach Updates suchen (nur, wenn eine Update-Quelle eingetragen ist) */
  autoUpdateCheck: boolean;
  /** Beim Minimieren aus der Taskleiste verschwinden und nur im Infobereich bleiben */
  minimizeToTray: boolean;
}

/** Namen stehen in den Sprachdateien unter `settings.accents`. */
export const ACCENTS = [
  { id: 'petrol', value: '#14707e' },
  { id: 'indigo', value: '#4b55c8' },
  { id: 'moss', value: '#4d7a2e' },
  { id: 'amber', value: '#b5650d' },
  { id: 'raspberry', value: '#b8336a' },
  { id: 'graphite', value: '#4a5258' },
] as const;

export const DEFAULT_TEMPLATE = '{name}';

const KEY = 'audio-konverter.settings.v1';

const CODECS: Codec[] = ['mp3', 'aac', 'opus', 'vorbis', 'flac', 'alac', 'wav', 'aiff'];

export const defaults: Settings = {
  language: 'system',
  mode: 'simple',
  theme: 'system',
  density: 'comfortable',
  accent: ACCENTS[0].value,
  format: 'mp3',
  presetByFormat: {},
  outputDir: null,
  template: DEFAULT_TEMPLATE,
  overwrite: false,
  subfolders: true,
  sortKey: 'added',
  sortDir: 'asc',
  autoUpdateCheck: true,
  minimizeToTray: false,
};

const oneOf =
  <T extends string>(...values: T[]) =>
  (v: unknown): v is T =>
    typeof v === 'string' && (values as string[]).includes(v);
const isBool = (v: unknown): v is boolean => typeof v === 'boolean';

/** Prüfregel je Feld: Nur passende Werte werden übernommen. */
const VALID: { [K in keyof Settings]: (v: unknown) => boolean } = {
  language: oneOf('system', 'de', 'en'),
  mode: oneOf('simple', 'advanced'),
  theme: oneOf('system', 'light', 'dark'),
  density: oneOf('comfortable', 'compact'),
  accent: (v) => typeof v === 'string' && /^#[0-9a-f]{6}$/i.test(v),
  format: oneOf(...CODECS),
  presetByFormat: (v) =>
    typeof v === 'object' &&
    v !== null &&
    !Array.isArray(v) &&
    Object.entries(v).every(([k, id]) => CODECS.includes(k as Codec) && typeof id === 'string'),
  outputDir: (v) => v === null || typeof v === 'string',
  template: (v) => typeof v === 'string' && v.length <= 500,
  overwrite: isBool,
  subfolders: isBool,
  sortKey: oneOf('added', 'name', 'sampleRate', 'bitsPerSample', 'bitRate'),
  sortDir: oneOf('asc', 'desc'),
  autoUpdateCheck: isBool,
  minimizeToTray: isBool,
};

/**
 * Übernimmt aus beliebigen Daten (localStorage, Importdatei) nur bekannte Felder
 * mit gültigen Werten. Alles andere wird still ignoriert.
 */
export function normalizeSettings(raw: unknown): Partial<Settings> {
  const result: Partial<Settings> = {};
  if (typeof raw !== 'object' || raw === null) return result;
  for (const key of Object.keys(VALID) as (keyof Settings)[]) {
    const value = (raw as Record<string, unknown>)[key];
    if (value !== undefined && VALID[key](value)) (result as Record<string, unknown>)[key] = value;
  }
  return result;
}

function load(): Settings {
  try {
    return { ...defaults, ...normalizeSettings(JSON.parse(localStorage.getItem(KEY) ?? '{}')) };
  } catch {
    return { ...defaults };
  }
}

export const settings = $state<Settings>(load());

export function persist() {
  try {
    localStorage.setItem(KEY, JSON.stringify(settings));
  } catch {
    /* Speicher nicht verfügbar – Einstellungen gelten nur für diese Sitzung */
  }
}

/** Übernimmt importierte Werte in die aktuellen Einstellungen. */
export function applySettings(values: Partial<Settings>) {
  Object.assign(settings, values);
}

/** Wirken Einstellungen, die im einfachen Modus nicht zu sehen sind? */
export function hiddenSettingsActive(): boolean {
  return settings.overwrite;
}

/** Aktuelles Preset: das zuletzt gewählte des Formats, sonst das erste. */
export function currentPreset<T extends { id: string; codec: Codec }>(presets: T[]): T | undefined {
  const forFormat = presets.filter((p) => p.codec === settings.format);
  return forFormat.find((p) => p.id === settings.presetByFormat[settings.format]) ?? forFormat[0];
}
