<script lang="ts">
  import { api, FORMATS } from '../api';
  import { describe, extension } from '../format';
  import { errorText, shortcut, t } from '../i18n/index.svelte';
  import { isActive, queue, type QueueItem } from '../queue.svelte';
  import { settings, type SortKey } from '../settings.svelte';
  import Dropdown from './Dropdown.svelte';
  import Icon from './Icon.svelte';
  import LevelMeter from './LevelMeter.svelte';

  function statusText(item: QueueItem): string {
    if (item.status === 'running' && item.progress > 0) return `${Math.floor(item.progress * 100)} %`;
    return t().queue.status[item.status];
  }

  /** Zweite Zeile bei Fehler oder Überspringen, in der aktuellen Sprache */
  function messageText(item: QueueItem): string {
    if (item.error) return errorText(item.error);
    if (item.status === 'skipped' && item.skip) {
      const format = FORMATS.find((f) => f.codec === item.skip!.codec)?.folder ?? item.skip.codec;
      return t().skip[item.skip.kind](format);
    }
    return '';
  }

  const SORT_KEYS: SortKey[] = ['added', 'name', 'sampleRate', 'bitsPerSample', 'bitRate'];

  // Zahlen starten absteigend (höchste zuerst), Name und Reihenfolge aufsteigend.
  function setSortKey(key: SortKey) {
    settings.sortKey = key;
    settings.sortDir = key === 'added' || key === 'name' ? 'asc' : 'desc';
  }

  const ascending = $derived(settings.sortDir === 'asc');

  const selectedCount = $derived(queue.selected.size);
  const someSelected = $derived(selectedCount > 0 && !queue.allSelected);

  function onRowClick(e: MouseEvent, item: QueueItem) {
    // Klicks auf Buttons/Checkboxen in der Zeile haben eigene Aktionen.
    if ((e.target as HTMLElement).closest('button, input')) return;
    queue.select(item.id, e.shiftKey ? 'range' : e.metaKey || e.ctrlKey ? 'toggle' : 'single');
  }

  function onCheckboxClick(e: MouseEvent, item: QueueItem) {
    queue.select(item.id, e.shiftKey ? 'range' : 'toggle');
    // Häkchen an den tatsächlichen Zustand angleichen (z. B. nach Bereichsauswahl)
    (e.currentTarget as HTMLInputElement).checked = queue.selected.has(item.id);
  }

  function toggleAll(e: Event) {
    if (queue.allSelected) queue.clearSelection();
    else queue.selectAll();
    (e.currentTarget as HTMLInputElement).checked = queue.allSelected;
  }

  /** `indeterminate` ist nur als DOM-Eigenschaft setzbar, nicht als Attribut. */
  function indeterminate(node: HTMLInputElement, value: boolean) {
    node.indeterminate = value;
    return {
      update(next: boolean) {
        node.indeterminate = next;
      },
    };
  }

  function onKeydown(e: KeyboardEvent) {
    // Nicht eingreifen, solange in ein Feld getippt wird oder ein Dialog offen ist.
    const target = e.target as HTMLElement;
    if (target.closest('input, select, textarea, [role="dialog"]')) return;

    if ((e.key === 'Delete' || e.key === 'Backspace') && selectedCount > 0) {
      e.preventDefault();
      queue.removeSelected();
    } else if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'a') {
      e.preventDefault();
      queue.selectAll();
    } else if (e.key === 'Escape' && selectedCount > 0) {
      queue.clearSelection();
    }
  }

  const showMeter = (i: QueueItem) => i.status === 'queued' || i.status === 'running' || i.status === 'done';
  const meterPhase = (i: QueueItem) => (i.status === 'done' ? 'done' : i.status === 'queued' ? 'waiting' : 'running');
</script>

<svelte:window onkeydown={onKeydown} />

<div class="list-wrap">
  <div class="list-head">
    <label class="count">
      <input
        type="checkbox"
        class="check"
        checked={queue.allSelected}
        use:indeterminate={someSelected}
        disabled={queue.selectableIds.length === 0}
        aria-label={t().queue.selectAll}
        title={t().queue.selectAllTitle(shortcut('mod', 'A'))}
        onchange={toggleAll}
      />
      {#if selectedCount > 0}
        {t().queue.selected(selectedCount)}
      {:else}
        {t().queue.files(queue.counts.total)}
        {#if queue.counts.done > 0}<span class="sub">{t().queue.doneCount(queue.counts.done)}</span>{/if}
      {/if}
    </label>
    <div class="sort">
      <span class="sort-label">{t().queue.sortLabel}</span>
      <Dropdown
        size="xs"
        label={t().queue.sortAria}
        value={settings.sortKey}
        options={SORT_KEYS.map((key) => ({ value: key, label: t().queue.sort[key] }))}
        onchange={setSortKey}
      />
      <button
        class="icon-btn"
        title={ascending ? t().queue.ascTitle : t().queue.descTitle}
        aria-label={ascending ? t().queue.ascAria : t().queue.descAria}
        onclick={() => (settings.sortDir = ascending ? 'desc' : 'asc')}
      >
        <Icon name={ascending ? 'arrow-up' : 'arrow-down'} />
      </button>
    </div>

    <div class="head-actions">
      {#if selectedCount > 0}
        <button class="btn quiet" onclick={() => queue.clearSelection()}>{t().queue.clearSelection}</button>
        <button class="btn quiet danger" onclick={() => queue.removeSelected()} title={t().queue.removeTitle(shortcut('del'))}>
          {t().queue.removeSelected(selectedCount)}
        </button>
      {:else}
        <button class="btn quiet" onclick={() => queue.clearFinished()} disabled={!queue.hasFinished}>
          {t().queue.clearFinished}
        </button>
        <button class="btn quiet" onclick={() => queue.clearAll()} disabled={queue.busy}>{t().queue.clearAll}</button>
      {/if}
    </div>
  </div>

  <ul class="list" role="listbox" aria-multiselectable="true" aria-label={t().queue.listAria}>
    {#each queue.view as item (item.id)}
      {@const selected = queue.selected.has(item.id)}
      {@const message = messageText(item)}
      <li
        class="row"
        class:selected
        data-status={item.status}
        role="option"
        aria-selected={selected}
        tabindex="-1"
        onclick={(e) => onRowClick(e, item)}
      >
        <input
          type="checkbox"
          class="check"
          checked={selected}
          disabled={isActive(item)}
          aria-label={t().queue.selectFile(item.fileName)}
          onclick={(e) => onCheckboxClick(e, item)}
        />
        <span class="badge">{extension(item.fileName) || '?'}</span>

        <div class="main">
          <div class="name" title={item.path}>{item.fileName}</div>
          {#if item.error}
            <div class="meta error" title={message}>{message}</div>
          {:else if message}
            <div class="meta note" title={message}>{message}</div>
          {:else if item.status === 'done' && item.output}
            <div class="meta" title={item.output}>{t().queue.saved(item.output)}</div>
          {:else if item.info}
            <div class="meta">
              {#each describe(item.info) as part}<span>{part}</span>{/each}
            </div>
          {/if}
          {#if showMeter(item)}
            <LevelMeter value={item.progress} phase={meterPhase(item)} />
          {/if}
        </div>

        <span class="status">
          {#if item.status === 'done'}<Icon name="check" />{:else if item.status === 'error' || item.status === 'unsupported'}<Icon name="alert" />{/if}
          {statusText(item)}
        </span>

        {#if item.status === 'queued' || item.status === 'running'}
          <button class="icon-btn" title={t().queue.cancel} aria-label={t().queue.cancel} onclick={() => api.cancel(item.id)}>
            <Icon name="stop" />
          </button>
        {:else}
          <button class="icon-btn" title={t().queue.remove} aria-label={t().queue.remove} onclick={() => queue.remove(item.id)}>
            <Icon name="close" />
          </button>
        {/if}
      </li>
    {/each}
  </ul>
</div>

<style>
  .list-wrap {
    height: 100%;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .list-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 12px;
    padding: 0 4px 8px;
  }
  .count {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding-left: 11px;
    font-weight: 600;
  }
  .check {
    width: 16px;
    height: 16px;
    margin: 0;
    accent-color: var(--accent);
    cursor: pointer;
  }
  .check:disabled {
    cursor: default;
    opacity: 0.35;
  }
  .sub {
    font-weight: 400;
    color: var(--muted);
  }
  .head-actions {
    display: flex;
    gap: 2px;
  }
  .sort {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
    color: var(--muted);
    font-size: 0.92em;
  }
  .sort .icon-btn {
    width: 28px;
    height: 28px;
  }
  .sort .icon-btn :global(svg) {
    width: 16px;
    height: 16px;
  }
  .head-actions .btn {
    height: 28px;
    padding: 0 10px;
    color: var(--muted);
  }
  .head-actions .btn.danger {
    color: var(--danger);
    font-weight: 600;
  }
  .list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    margin: 0;
    padding: 0;
    list-style: none;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
  }
  .row {
    display: grid;
    grid-template-columns: 16px 52px minmax(0, 1fr) auto 30px;
    align-items: center;
    gap: 12px;
    padding: var(--row-y) 10px var(--row-y) 14px;
    border-bottom: 1px solid var(--line);
    outline: none;
  }
  .row.selected {
    background: var(--accent-soft);
  }
  .row:last-child {
    border-bottom: 0;
  }
  .badge {
    justify-self: start;
    padding: 2px 7px;
    border-radius: 5px;
    background: var(--surface-sunk);
    color: var(--muted);
    font-size: 0.78em;
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: lowercase;
  }
  .main {
    min-width: 0;
    display: grid;
    gap: 4px;
  }
  .name,
  .meta {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .name {
    font-weight: 500;
  }
  .meta {
    display: flex;
    gap: 14px;
    color: var(--muted);
    font-size: 0.88em;
    font-variant-numeric: tabular-nums;
  }
  .meta.error {
    display: block;
    color: var(--danger);
  }
  .meta.note {
    display: block;
  }
  [data-status='skipped'] .status {
    color: var(--muted);
  }
  .status {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    min-width: 84px;
    justify-content: flex-end;
    color: var(--muted);
    font-size: 0.9em;
    font-variant-numeric: tabular-nums;
  }
  .status :global(svg) {
    width: 15px;
    height: 15px;
  }
  [data-status='running'] .status {
    color: var(--accent);
    font-weight: 600;
  }
  [data-status='done'] .status {
    color: var(--ok);
  }
  [data-status='error'] .status,
  [data-status='unsupported'] .status {
    color: var(--danger);
  }
  [data-status='unsupported'] .name {
    color: var(--muted);
  }

  @media (max-width: 620px) {
    .row {
      grid-template-columns: 16px minmax(0, 1fr) auto 30px;
    }
    .badge {
      display: none;
    }
    .status {
      min-width: 0;
    }
  }
</style>
