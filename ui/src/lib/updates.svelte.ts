// Update-Prüfung – vorbereitet, aber noch nicht aktiv.
//
// Aktivieren (einfache Variante): UPDATE_URL auf die GitHub-API des Projekts setzen, z. B.
//   'https://api.github.com/repos/<nutzer>/<repo>/releases/latest'
// Die App vergleicht dann `tag_name` (z. B. "v0.2.0") mit der eigenen Version und
// bietet einen Link zur Release-Seite an. Herunterladen und Installieren macht der Nutzer.
//
// Später (Auto-Updater): tauri-plugin-updater einbinden, Signierschlüssel erzeugen und
// `checkForUpdate` durch `check()` aus @tauri-apps/plugin-updater ersetzen. Oberfläche und
// Einstellungen bleiben gleich.

export const UPDATE_URL: string | null = null;

/** Ist eine Update-Quelle eingetragen? Sonst sind Schalter und Button gesperrt. */
export const updatesEnabled = UPDATE_URL !== null;

export interface UpdateInfo {
  version: string;
  /** Seite, auf der die neue Version heruntergeladen werden kann */
  url: string;
}

export type UpdateStatus =
  | { state: 'idle' }
  | { state: 'checking' }
  | { state: 'current' }
  | { state: 'available'; info: UpdateInfo }
  | { state: 'error'; message: string };

/** Ergebnis der letzten Prüfung, geteilt zwischen Start-Prüfung und Einstellungen. */
export const update = $state<{ status: UpdateStatus }>({ status: { state: 'idle' } });

const LAST_CHECK_KEY = 'audio-konverter.update-check.v1';
const DAY_MS = 24 * 60 * 60 * 1000;

/** Vergleicht Versionen wie "0.10.2" und "v0.9.0"; Vorabversionen (-beta) zählen als älter. */
export function isNewer(candidate: string, current: string): boolean {
  const parse = (v: string) => {
    const [main, pre] = v.trim().replace(/^v/i, '').split('-', 2);
    return { nums: main.split('.').map((n) => parseInt(n, 10) || 0), pre: pre ?? null };
  };
  const a = parse(candidate);
  const b = parse(current);
  for (let i = 0; i < Math.max(a.nums.length, b.nums.length); i++) {
    const diff = (a.nums[i] ?? 0) - (b.nums[i] ?? 0);
    if (diff !== 0) return diff > 0;
  }
  // Gleiche Nummer: endgültige Version ist neuer als eine Vorabversion.
  return a.pre === null && b.pre !== null;
}

/** Fragt die Update-Quelle ab. Liefert null, wenn die App aktuell ist. */
async function fetchLatest(currentVersion: string): Promise<UpdateInfo | null> {
  if (!UPDATE_URL) return null;
  const response = await fetch(UPDATE_URL, { headers: { Accept: 'application/vnd.github+json' } });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  const release = (await response.json()) as { tag_name?: string; html_url?: string; draft?: boolean; prerelease?: boolean };
  if (!release.tag_name || release.draft || release.prerelease) return null;
  const version = release.tag_name.replace(/^v/i, '');
  return isNewer(version, currentVersion) ? { version, url: release.html_url ?? UPDATE_URL } : null;
}

/** Prüft auf Updates und legt das Ergebnis in `update.status` ab. */
export async function checkForUpdate(currentVersion: string): Promise<UpdateStatus> {
  if (!updatesEnabled) return update.status;
  update.status = { state: 'checking' };
  try {
    const info = await fetchLatest(currentVersion);
    update.status = info ? { state: 'available', info } : { state: 'current' };
    markChecked();
  } catch (e) {
    update.status = { state: 'error', message: e instanceof Error ? e.message : String(e) };
  }
  return update.status;
}

/** Automatische Prüfung höchstens einmal am Tag. */
export function dueForAutoCheck(): boolean {
  if (!updatesEnabled) return false;
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
