// Updates über tauri-plugin-updater.
//
// Quelle: latest.json am neuesten GitHub-Release (siehe plugins.updater in
// app/tauri.conf.json). Die Datei und die Signaturen erzeugt der Release-Workflow;
// Updates werden nur installiert, wenn ihre Signatur zum öffentlichen Schlüssel passt.
//
// Portable Windows-Version, Linux-.deb und Entwicklungsbuilds können sich nicht selbst
// ersetzen (api.installInfo). Dort gibt es stattdessen einen Link zur Download-Seite.
import { relaunch } from '@tauri-apps/plugin-process';
import { check, type Update } from '@tauri-apps/plugin-updater';

/** Release-Seite für alle, die nicht direkt aus der App aktualisieren können */
export const RELEASES_URL = 'https://github.com/Olcay91/Audio-Konverter/releases/latest';

/** Update-Prüfung ist eingerichtet (Schalter und Button in den Einstellungen aktiv). */
export const updatesEnabled = true;

export type UpdateStatus =
  | { state: 'idle' }
  | { state: 'checking' }
  | { state: 'current' }
  | { state: 'available'; version: string; notes: string }
  /** progress: 0–1, null solange die Größe unbekannt ist */
  | { state: 'downloading'; version: string; progress: number | null }
  | { state: 'installing'; version: string }
  /** kind: offline = Server nicht erreichbar, sonst sonstiger Fehler mit Meldung */
  | { state: 'error'; kind: 'offline' | 'other'; message: string };

/** Zustand, geteilt zwischen Start-Prüfung und Einstellungen. */
export const update = $state<{ status: UpdateStatus }>({ status: { state: 'idle' } });

/** Gefundenes Update; nötig für die Installation. */
let pending: Update | null = null;

const LAST_CHECK_KEY = 'audio-konverter.update-check.v1';
const DAY_MS = 24 * 60 * 60 * 1000;

const message = (e: unknown) => (e instanceof Error ? e.message : String(e));

/**
 * Ordnet Meldungen des Updaters ein:
 * - „Could not fetch a valid release JSON“: Es gibt (noch) kein veröffentlichtes Release mit
 *   latest.json. Für Nutzer heißt das schlicht: keine neuere Version.
 * - Netzwerkfehler (reqwest): Server nicht erreichbar.
 */
function classify(e: unknown): UpdateStatus {
  const text = message(e);
  if (/valid release JSON/i.test(text)) return { state: 'current' };
  if (/error sending request|dns|connect|timed? ?out|network|offline/i.test(text)) {
    return { state: 'error', kind: 'offline', message: text };
  }
  return { state: 'error', kind: 'other', message: text };
}

/**
 * Prüft auf Updates und legt das Ergebnis in `update.status` ab.
 * `silent` (automatische Prüfung beim Start): Fehler werden nicht angezeigt.
 */
export async function checkForUpdate(options: { silent?: boolean } = {}): Promise<UpdateStatus> {
  update.status = { state: 'checking' };
  try {
    pending = await check();
    update.status = pending
      ? { state: 'available', version: pending.version, notes: pending.body ?? '' }
      : { state: 'current' };
    markChecked();
  } catch (e) {
    pending = null;
    const result = classify(e);
    if (result.state === 'error') console.warn('Update-Prüfung:', result.message);
    else markChecked();
    update.status = result.state === 'error' && options.silent ? { state: 'idle' } : result;
  }
  return update.status;
}

/**
 * Lädt das gefundene Update, installiert es und startet die App neu.
 * Unter Windows beendet der Installer die App selbst und startet sie danach wieder.
 */
export async function installUpdate(): Promise<void> {
  if (!pending) return;
  const version = pending.version;
  let total = 0;
  let received = 0;
  update.status = { state: 'downloading', version, progress: null };
  try {
    await pending.downloadAndInstall((event) => {
      if (event.event === 'Started') {
        total = event.data.contentLength ?? 0;
      } else if (event.event === 'Progress') {
        received += event.data.chunkLength;
        update.status = { state: 'downloading', version, progress: total > 0 ? received / total : null };
      } else if (event.event === 'Finished') {
        update.status = { state: 'installing', version };
      }
    });
    await relaunch();
  } catch (e) {
    const result = classify(e);
    update.status = result.state === 'error' ? result : { state: 'error', kind: 'other', message: message(e) };
  }
}

/** Automatische Prüfung höchstens einmal am Tag. */
export function dueForAutoCheck(): boolean {
  try {
    const last = Number(localStorage.getItem(LAST_CHECK_KEY) ?? 0);
    return !(last > 0 && Date.now() - last < DAY_MS);
  } catch {
    return true;
  }
}

function markChecked() {
  try {
    localStorage.setItem(LAST_CHECK_KEY, String(Date.now()));
  } catch {
    /* ohne Speicher wird beim nächsten Start erneut geprüft */
  }
}
