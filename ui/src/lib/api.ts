// Typen und Aufrufe der Rust-Befehle (app/src/commands.rs).
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface MediaInfo {
  formatName: string;
  durationSecs: number | null;
  codec: string;
  sampleRate: number | null;
  channels: number | null;
  bitRate: number | null;
  bitsPerSample: number | null;
  hasCover: boolean;
  tags: Record<string, string>;
}

export type Codec = 'mp3' | 'aac' | 'opus' | 'vorbis' | 'flac' | 'alac' | 'wav' | 'aiff';

export type Rate =
  | { mode: 'quality'; value: number }
  | { mode: 'bitrate'; kbps: number }
  | { mode: 'lossless'; compression?: number | null };

export interface Preset {
  id: string;
  name: string;
  description: string;
  codec: Codec;
  rate: Rate;
  sampleRate: number | null;
  channels: number | null;
  bitDepth: number | null;
  builtin: boolean;
}

/** Fehler mit stabilem Code; der Text kommt aus den Sprachdateien (siehe i18n/errorText). */
export interface UiError {
  code: string;
  /** Sprachunabhängiger Zusatz, z. B. eine Meldung von ffmpeg oder des Systems */
  detail?: string;
}

export interface Environment {
  appVersion: string;
  ffmpegPath: string;
  ffmpegVersion: string | null;
  ffmpegError: UiError | null;
  parallelJobs: number;
}

export interface AddedItem {
  id: number;
  path: string;
  fileName: string;
  info: MediaInfo | null;
  error: UiError | null;
}

export type JobStatus = 'queued' | 'running' | 'done' | 'error' | 'cancelled' | 'skipped';

/** Warum eine Datei übersprungen wurde (command.rs: SkipReason) */
export interface SkipReason {
  kind: 'sameLossless' | 'notSmaller';
  codec: Codec;
}

export interface JobUpdate {
  id: number;
  status: JobStatus;
  progress: number;
  output: string | null;
  error: UiError | null;
  skip: SkipReason | null;
}

export interface BatchSummary {
  done: number;
  failed: number;
  cancelled: number;
  skipped: number;
}

export interface StartRequest {
  ids: number[];
  presetId: string;
  outputDir: string | null;
  template: string | null;
  /** Vorhandene Zieldateien ersetzen statt " (2)" anzuhängen */
  overwrite: boolean;
  /** Eigener Zielordner: Struktur nachbilden. Ordner der Originaldatei: Unterordner mit Formatnamen. */
  subfolders: boolean;
  /** Sprache der Systembenachrichtigung */
  lang: 'de' | 'en';
  /** Ersatz für fehlende Tags in der Dateinamen-Vorlage */
  fallbacks: { artist: string; album: string };
}

/** Fortschritt beim Hinzufügen (Event `add-progress`) */
export interface AddProgress {
  requestId: number;
  /** scanning: Ordner werden durchsucht · reading: ffprobe liest die Dateien */
  phase: 'scanning' | 'reading';
  /** Bisher gefundene Audiodateien */
  found: number;
  done: number;
  total: number;
}

export interface ImportResult {
  presets: Preset[];
  added: number;
  replaced: number;
  skipped: number;
  invalid: number;
  /** ID aus der Datei → ID in der App */
  idMap: Record<string, string>;
}

export const LOSSLESS: Codec[] = ['flac', 'alac', 'wav', 'aiff'];

/** `folder` muss zu Codec::label in core/src/preset.rs passen. */
export const FORMATS: { codec: Codec; label: string; folder: string }[] = [
  { codec: 'mp3', label: 'MP3', folder: 'MP3' },
  { codec: 'aac', label: 'AAC (M4A)', folder: 'AAC' },
  { codec: 'opus', label: 'Opus', folder: 'Opus' },
  { codec: 'vorbis', label: 'Ogg Vorbis', folder: 'Ogg Vorbis' },
  { codec: 'flac', label: 'FLAC', folder: 'FLAC' },
  { codec: 'alac', label: 'ALAC (M4A)', folder: 'ALAC' },
  { codec: 'wav', label: 'WAV', folder: 'WAV' },
  { codec: 'aiff', label: 'AIFF', folder: 'AIFF' },
];

export const BIT_DEPTHS = [16, 24];

/** Muss zu Codec::sample_rates in core/src/preset.rs passen. Opus arbeitet immer mit 48 kHz. */
export function sampleRatesFor(codec: Codec): number[] {
  switch (codec) {
    case 'opus':
      return [];
    case 'mp3':
      return [32000, 44100, 48000];
    case 'aac':
    case 'vorbis':
      return [44100, 48000, 88200, 96000];
    default:
      return [44100, 48000, 88200, 96000, 176400, 192000];
  }
}

/** Auswahl an Bitraten für den Preset-Editor (kbit/s). */
export function bitratesFor(codec: Codec): number[] {
  switch (codec) {
    case 'mp3':
      return [128, 160, 192, 224, 256, 320];
    case 'aac':
      return [96, 128, 160, 192, 256, 320];
    case 'opus':
      return [64, 96, 128, 160, 192, 256];
    case 'vorbis':
      return [96, 128, 160, 192, 256, 320];
    default:
      return [];
  }
}

export const MP3_VBR_KBPS = [245, 225, 190, 175, 165, 130, 115, 100, 85, 65];
export const VORBIS_KBPS: Record<number, number> = { 10: 500, 9: 320, 8: 256, 7: 224, 6: 192, 5: 160, 4: 128, 3: 112, 2: 96, 1: 80, 0: 64 };

/** Qualitätsstufen (VBR), beste zuerst, mit ungefährer Bitrate. Leer, wenn das Format keine hat. */
export function qualitiesFor(codec: Codec): { value: number; label: string; kbps: number }[] {
  if (codec === 'mp3') return MP3_VBR_KBPS.map((kbps, v) => ({ value: v, label: `V${v}`, kbps }));
  if (codec === 'vorbis')
    return [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0].map((q) => ({ value: q, label: `q${q}`, kbps: VORBIS_KBPS[q] }));
  return [];
}

export const AUDIO_EXTENSIONS = [
  'aac', 'ac3', 'aif', 'aifc', 'aiff', 'alac', 'amr', 'ape', 'au', 'caf', 'dff', 'dsf', 'dts',
  'flac', 'm4a', 'm4b', 'mka', 'mp2', 'mp3', 'mpc', 'oga', 'ogg', 'opus', 'tta', 'w64', 'wav',
  'wma', 'wv',
];

export const api = {
  environment: () => invoke<Environment>('environment'),
  presets: () => invoke<Preset[]>('presets'),
  savePreset: (preset: Preset) => invoke<{ id: string; presets: Preset[] }>('save_preset', { preset }),
  deletePreset: (id: string) => invoke<Preset[]>('delete_preset', { id }),
  /** Eigene Presets aus einer Einstellungsdatei übernehmen */
  importPresets: (presets: Preset[], replace: boolean) => invoke<ImportResult>('import_presets', { presets, replace }),
  readTextFile: (path: string) => invoke<string>('read_text_file', { path }),
  writeTextFile: (path: string, contents: string) => invoke<void>('write_text_file', { path, contents }),
  /** Tray-Symbol ein-/ausschalten; die Menütexte kommen aus der Sprachdatei. */
  setMinimizeToTray: (enabled: boolean, labels: { show: string; quit: string; tooltip: string }) =>
    invoke<void>('set_minimize_to_tray', { enabled, labels }),
  /** Mit `requestId` kommen unterwegs `add-progress`-Events. */
  addPaths: (paths: string[], requestId?: number) =>
    invoke<{ items: AddedItem[]; skipped: number }>('add_paths', { paths, requestId }),
  removeItems: (ids: number[]) => invoke<void>('remove_items', { ids }),
  /** Liefert die Anzahl gestarteter Jobs (0 = nichts zu tun). */
  start: (request: StartRequest) => invoke<number>('start', { request }),
  cancel: (id: number) => invoke<void>('cancel', { id }),
  cancelAll: () => invoke<void>('cancel_all'),
  onJobUpdate: (cb: (u: JobUpdate) => void) => listen<JobUpdate>('job-update', (e) => cb(e.payload)),
  onAddProgress: (cb: (p: AddProgress) => void) => listen<AddProgress>('add-progress', (e) => cb(e.payload)),
  onBatchFinished: (cb: (s: BatchSummary) => void) =>
    listen<BatchSummary>('batch-finished', (e) => cb(e.payload)),
};
