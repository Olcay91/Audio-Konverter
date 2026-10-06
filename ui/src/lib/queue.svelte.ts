import { SvelteSet } from 'svelte/reactivity';
import { api, type AddProgress, type AddedItem, type JobStatus, type JobUpdate, type MediaInfo, type SkipReason, type UiError } from './api';
import { settings } from './settings.svelte';

const collator = new Intl.Collator('de', { numeric: true, sensitivity: 'base' });

export type ItemStatus = 'ready' | 'unsupported' | JobStatus;

export interface QueueItem {
  id: number;
  path: string;
  fileName: string;
  info: MediaInfo | null;
  status: ItemStatus;
  progress: number;
  output: string | null;
  /** Wird erst beim Anzeigen übersetzt, damit ein Sprachwechsel auch hier greift. */
  error: UiError | null;
  skip: SkipReason | null;
}

const ACTIVE: ItemStatus[] = ['queued', 'running'];
const FINISHED: ItemStatus[] = ['done', 'error', 'cancelled', 'unsupported', 'skipped'];
// Übersprungene bleiben startbar: Mit einem anderen Preset ergibt die Konvertierung evtl. Sinn.
const STARTABLE: ItemStatus[] = ['ready', 'error', 'cancelled', 'skipped'];

function fromAdded(a: AddedItem): QueueItem {
  return {
    id: a.id,
    path: a.path,
    fileName: a.fileName,
    info: a.info,
    status: a.info ? 'ready' : 'unsupported',
    progress: 0,
    output: null,
    error: a.error,
    skip: null,
  };
}

export const isActive = (i: QueueItem) => ACTIVE.includes(i.status);

class Queue {
  items = $state<QueueItem[]>([]);

  /** Laufende Einlesevorgänge nach Anfrage-ID (es kann mehrere gleichzeitig geben) */
  private reading = $state<Record<number, AddProgress>>({});
  private nextRequest = 1;

  get adding() {
    return Object.keys(this.reading).length > 0;
  }

  /**
   * Fortschritt aller laufenden Einlesevorgänge zusammengefasst. Solange noch
   * irgendwo Ordner durchsucht werden, gilt die Phase „scanning“.
   */
  get addProgress(): Omit<AddProgress, 'requestId'> | null {
    const all = Object.values(this.reading);
    if (all.length === 0) return null;
    const sum = (key: 'found' | 'done' | 'total') => all.reduce((n, p) => n + p[key], 0);
    const phase = all.some((p) => p.phase === 'scanning') ? 'scanning' : 'reading';
    return { phase, found: sum('found'), done: sum('done'), total: sum('total') };
  }

  /** Fortschritts-Event aus Rust; verspätete Events beendeter Anfragen werden ignoriert. */
  applyProgress(p: AddProgress) {
    if (p.requestId in this.reading) this.reading[p.requestId] = p;
  }

  /**
   * Liste in der gewählten Sortierung. Dateien ohne den jeweiligen Wert
   * (z. B. Bit-Tiefe bei MP3) stehen immer am Ende.
   */
  view = $derived.by(() => {
    const { sortKey, sortDir } = settings;
    if (sortKey === 'added') return sortDir === 'asc' ? this.items : [...this.items].reverse();

    const dir = sortDir === 'asc' ? 1 : -1;
    const value = (i: QueueItem): string | number | null =>
      sortKey === 'name' ? i.fileName : (i.info?.[sortKey] ?? null);

    return [...this.items].sort((a, b) => {
      const va = value(a);
      const vb = value(b);
      if (va === null || vb === null) {
        if (va === vb) return collator.compare(a.fileName, b.fileName);
        return va === null ? 1 : -1;
      }
      const diff = typeof va === 'string' ? collator.compare(va, vb as string) : va - (vb as number);
      return diff !== 0 ? diff * dir : collator.compare(a.fileName, b.fileName) || a.id - b.id;
    });
  });

  /** Markierte Einträge. Laufende und wartende Jobs lassen sich nicht markieren. */
  selected = new SvelteSet<number>();
  /** Ausgangspunkt für Bereichsauswahl mit Umschalt */
  private anchor: number | null = null;

  get selectableIds() {
    return this.items.filter((i) => !isActive(i)).map((i) => i.id);
  }

  get allSelected() {
    const ids = this.selectableIds;
    return ids.length > 0 && ids.every((id) => this.selected.has(id));
  }

  /**
   * single: nur diesen Eintrag markieren · toggle: hinzufügen/entfernen (Strg/⌘)
   * range: Bereich ab dem letzten Klick (Umschalt)
   */
  select(id: number, mode: 'single' | 'toggle' | 'range') {
    const item = this.items.find((i) => i.id === id);
    if (!item || isActive(item)) return;

    if (mode === 'range' && this.anchor !== null) {
      // Bereich so, wie er auf dem Bildschirm zu sehen ist (also sortiert)
      const from = this.view.findIndex((i) => i.id === this.anchor);
      const to = this.view.findIndex((i) => i.id === id);
      if (from !== -1 && to !== -1) {
        this.selected.clear();
        for (const i of this.view.slice(Math.min(from, to), Math.max(from, to) + 1)) {
          if (!isActive(i)) this.selected.add(i.id);
        }
        return;
      }
    }
    if (mode === 'toggle') {
      if (this.selected.has(id)) this.selected.delete(id);
      else this.selected.add(id);
    } else {
      this.selected.clear();
      this.selected.add(id);
    }
    this.anchor = id;
  }

  selectAll() {
    for (const id of this.selectableIds) this.selected.add(id);
  }

  clearSelection() {
    this.selected.clear();
    this.anchor = null;
  }

  async removeSelected() {
    await this.removeIds([...this.selected]);
  }

  private async removeIds(ids: number[]) {
    const removable = new Set(this.items.filter((i) => ids.includes(i.id) && !isActive(i)).map((i) => i.id));
    if (removable.size === 0) return;
    await api.removeItems([...removable]);
    this.items = this.items.filter((i) => !removable.has(i.id));
    for (const id of removable) this.selected.delete(id);
    if (this.anchor !== null && removable.has(this.anchor)) this.anchor = null;
  }

  get busy() {
    return this.items.some((i) => ACTIVE.includes(i.status));
  }

  /** Einträge, die gestartet werden können (auch erneut, z. B. nach Fehler) */
  get startableItems() {
    return this.items.filter((i) => STARTABLE.includes(i.status));
  }

  get startableIds() {
    return this.items.filter((i) => STARTABLE.includes(i.status)).map((i) => i.id);
  }

  get hasFinished() {
    return this.items.some((i) => FINISHED.includes(i.status));
  }

  get counts() {
    const c = { total: this.items.length, done: 0, active: 0, failed: 0 };
    for (const i of this.items) {
      if (i.status === 'done') c.done++;
      else if (ACTIVE.includes(i.status)) c.active++;
      else if (i.status === 'error' || i.status === 'unsupported') c.failed++;
    }
    return c;
  }

  /** Fügt Dateien/Ordner hinzu und meldet, was dabei passiert ist. */
  async add(paths: string[]): Promise<{ added: number; unsupported: number; skipped: number }> {
    if (paths.length === 0) return { added: 0, unsupported: 0, skipped: 0 };
    const requestId = this.nextRequest++;
    this.reading[requestId] = { requestId, phase: 'scanning', found: 0, done: 0, total: 0 };
    try {
      const result = await api.addPaths(paths, requestId);
      // Dateien ohne Audio merkt sich das Backend nicht, daher hier doppelte vermeiden.
      const known = new Set(this.items.map((i) => i.path));
      const fresh = result.items.filter((a) => a.info || !known.has(a.path));
      this.items.push(...fresh.map(fromAdded));
      const unsupported = fresh.filter((a) => !a.info).length;
      return {
        added: fresh.length - unsupported,
        unsupported,
        skipped: result.skipped + (result.items.length - fresh.length),
      };
    } finally {
      delete this.reading[requestId];
    }
  }

  async remove(id: number) {
    await this.removeIds([id]);
  }

  async clearFinished() {
    await this.removeIds(this.items.filter((i) => FINISHED.includes(i.status)).map((i) => i.id));
  }

  async clearAll() {
    await this.removeIds(this.selectableIds);
  }

  apply(u: JobUpdate) {
    const item = this.items.find((i) => i.id === u.id);
    if (!item) return;
    item.status = u.status;
    item.progress = u.progress;
    if (u.output) item.output = u.output;
    item.error = u.error;
    item.skip = u.skip;
    // Ein gestarteter Job kann nicht mehr entfernt werden, also auch nicht markiert bleiben.
    if (isActive(item)) this.selected.delete(item.id);
  }
}

export const queue = new Queue();
