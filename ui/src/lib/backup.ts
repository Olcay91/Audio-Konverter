// Einstellungsdatei für Export und Import: Einstellungen plus eigene Presets.
import type { Preset } from './api';
import { normalizeSettings, type Settings } from './settings.svelte';

/** Kennung im Dateiinhalt, damit fremde JSON-Dateien erkannt werden */
const APP_ID = 'audio-konverter';
/** Bei inkompatiblen Änderungen erhöhen; ältere Versionen bleiben importierbar. */
export const BACKUP_VERSION = 1;

/** Gerätegebundene Werte, die nicht exportiert werden. */
const MACHINE_SPECIFIC: (keyof Settings)[] = ['outputDir'];

export interface Backup {
  app: typeof APP_ID;
  version: number;
  appVersion: string;
  exportedAt: string;
  settings: Partial<Settings>;
  presets: Preset[];
}

/** Fehler beim Einlesen; `code` passt zu den Fehlertexten in i18n (errors.*). */
export class BackupError extends Error {
  constructor(public code: 'backupInvalidJson' | 'backupNotABackup' | 'backupNewerVersion') {
    super(code);
  }
}

export function createBackup(settings: Settings, presets: Preset[], appVersion: string): string {
  // Kopie, damit das Entfernen gerätegebundener Werte die Einstellungen nicht ändert
  const exported: Partial<Settings> = JSON.parse(JSON.stringify(settings));
  for (const key of MACHINE_SPECIFIC) delete exported[key];
  const backup: Backup = {
    app: APP_ID,
    version: BACKUP_VERSION,
    appVersion,
    exportedAt: new Date().toISOString(),
    settings: exported,
    presets: presets.filter((p) => !p.builtin).map((p) => ({ ...p, builtin: false })),
  };
  return JSON.stringify(backup, null, 2);
}

/** Grobe Formprüfung; die genaue Prüfung macht Rust beim Import (Preset::validate). */
function looksLikePreset(p: unknown): p is Preset {
  if (typeof p !== 'object' || p === null) return false;
  const o = p as Record<string, unknown>;
  return typeof o.name === 'string' && typeof o.codec === 'string' && typeof o.rate === 'object' && o.rate !== null;
}

export function parseBackup(text: string): Backup {
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    throw new BackupError('backupInvalidJson');
  }
  if (typeof raw !== 'object' || raw === null || (raw as Record<string, unknown>).app !== APP_ID) {
    throw new BackupError('backupNotABackup');
  }
  const o = raw as Record<string, unknown>;
  const version = typeof o.version === 'number' ? o.version : 0;
  if (version > BACKUP_VERSION) throw new BackupError('backupNewerVersion');

  const settings = normalizeSettings(o.settings);
  for (const key of MACHINE_SPECIFIC) delete settings[key];
  const presets = (Array.isArray(o.presets) ? o.presets : []).filter(looksLikePreset).map((p) => ({
    id: typeof p.id === 'string' ? p.id : '',
    name: p.name,
    description: typeof p.description === 'string' ? p.description : '',
    codec: p.codec,
    rate: p.rate,
    sampleRate: p.sampleRate ?? null,
    channels: p.channels ?? null,
    bitDepth: p.bitDepth ?? null,
    builtin: false,
  }));

  return {
    app: APP_ID,
    version,
    appVersion: typeof o.appVersion === 'string' ? o.appVersion : '?',
    exportedAt: typeof o.exportedAt === 'string' ? o.exportedAt : '',
    settings,
    presets,
  };
}

/** Presets der Datei, die genauso heißen wie vorhandene (gleiches Format). */
export function conflictingPresets(incoming: Preset[], existing: Preset[]): Preset[] {
  const key = (p: Preset) => `${p.codec}\u0000${p.name.trim().toLowerCase()}`;
  const known = new Set(existing.map(key));
  return incoming.filter((p) => known.has(key(p)));
}
