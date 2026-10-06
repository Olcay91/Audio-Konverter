// Sprachauswahl und Hilfsfunktionen. Texte stehen in de.ts und en.ts.
//
// Verwendung: `t().bar.convert`. Weil t() die Einstellung `language` liest,
// aktualisieren sich Vorlagen und $derived-Werte beim Sprachwechsel von selbst.
import type { Preset, UiError } from '../api';
import { settings } from '../settings.svelte';
import { de, type Messages } from './de';
import { en } from './en';

export type Lang = 'de' | 'en';

/** Systemsprache; alles außer Deutsch fällt auf Englisch zurück. */
export function systemLang(): Lang {
  return (navigator.language || '').toLowerCase().startsWith('de') ? 'de' : 'en';
}

export function lang(): Lang {
  return settings.language === 'system' ? systemLang() : settings.language;
}

export function t(): Messages {
  return lang() === 'en' ? en : de;
}

/** Locale für Zahlen und Datumsangaben */
export function locale(): string {
  return lang() === 'en' ? 'en-US' : 'de-DE';
}

export const isMac = /Mac|iPhone|iPad/i.test(navigator.platform || navigator.userAgent);

/** Tastenkürzel passend zur Plattform, z. B. shortcut('mod', 'O') → „Strg + O“ bzw. „⌘ O“ */
export function shortcut(...keys: string[]): string {
  if (isMac) {
    const symbols: Record<string, string> = { mod: '⌘', shift: '⇧', del: '⌫' };
    return keys.map((k) => symbols[k] ?? k).join(' ');
  }
  const names = t().keys as Record<string, string>;
  return keys.map((k) => names[k] ?? k).join(' + ');
}

/** Text zu einem Fehler aus Rust (UiError) oder einer anderen Ausnahme. */
export function errorText(e: unknown): string {
  const errors = t().errors;
  if (typeof e === 'object' && e !== null && 'code' in e) {
    const { code, detail } = e as UiError;
    const fn = errors[code];
    return fn ? fn(detail ?? '') : errors.unknown(detail ? `${code}: ${detail}` : code);
  }
  return e instanceof Error ? e.message : String(e);
}

/** Übersetzung eines Standard-Presets, falls die Sprache eine hat (Deutsch: presets/default.toml). */
function builtinText(p: Preset) {
  return p.builtin ? t().presets.names[p.id] : undefined;
}

/** Name eines Presets; Standard-Presets werden übersetzt. */
export function presetName(p: Preset): string {
  return builtinText(p)?.name ?? p.name;
}

export function presetDescription(p: Preset): string {
  const entry = builtinText(p);
  return entry ? (entry.description ?? '') : p.description;
}
