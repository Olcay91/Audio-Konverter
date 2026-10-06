<script lang="ts">
  import type { Preset } from '../api';
  import { presetSummary } from '../format';
  import { presetDescription, presetName, t } from '../i18n/index.svelte';
  import Icon from './Icon.svelte';
  import PresetEditor from './PresetEditor.svelte';

  let {
    presets,
    current,
    onselect,
    onchange,
  }: {
    /** Presets des gewählten Formats */
    presets: Preset[];
    current: Preset | undefined;
    onselect: (id: string) => void;
    /** Liste aller Presets hat sich geändert (gespeichert oder gelöscht) */
    onchange: (all: Preset[]) => void;
  } = $props();

  let open = $state(false);
  let editing = $state<Preset | null>(null);
  let root = $state<HTMLDivElement>();

  const builtin = $derived(presets.filter((p) => p.builtin));
  const own = $derived(presets.filter((p) => !p.builtin));

  function choose(p: Preset) {
    onselect(p.id);
    open = false;
  }

  // Neues Preset startet mit den Werten der aktuellen Auswahl.
  function createNew() {
    if (!current) return;
    editing = { ...$state.snapshot(current), id: '', name: '', description: '', builtin: false };
    open = false;
  }

  function edit(p: Preset) {
    editing = $state.snapshot(p);
    open = false;
  }

  function onWindowClick(e: MouseEvent) {
    if (open && root && !root.contains(e.target as Node)) open = false;
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={(e) => e.key === 'Escape' && (open = false)} />

<div class="picker" bind:this={root}>
  <button
    class="select trigger"
    aria-haspopup="listbox"
    aria-expanded={open}
    title={current ? presetSummary(current) : ''}
    onclick={() => (open = !open)}
  >
    <span class="current">{current ? presetName(current) : t().presets.none}</span>
    <Icon name="chevron" />
  </button>

  {#if open}
    <div class="menu" role="listbox" aria-label={t().presets.listLabel}>
      <div class="group">{t().presets.builtin}</div>
      {#each builtin as p (p.id)}{@render row(p)}{/each}

      {#if own.length > 0}
        <div class="group">{t().presets.own}</div>
        {#each own as p (p.id)}{@render row(p)}{/each}
      {/if}

      <button class="new" onclick={createNew}>
        <Icon name="plus" /> {t().presets.createNew}
      </button>
    </div>
  {/if}
</div>

{#snippet row(p: Preset)}
  <div class="item" class:selected={p.id === current?.id}>
    <button
      class="choose"
      role="option"
      aria-selected={p.id === current?.id}
      title={presetDescription(p)}
      onclick={() => choose(p)}
    >
      <span class="name">{presetName(p)}</span>
      <span class="summary">{presetSummary(p)}</span>
    </button>
    {#if !p.builtin}
      <button class="icon-btn" title={t().presets.edit} aria-label={t().presets.editNamed(p.name)} onclick={() => edit(p)}>
        <Icon name="edit" />
      </button>
    {/if}
  </div>
{/snippet}

{#if editing}
  <PresetEditor
    preset={editing}
    onclose={() => (editing = null)}
    onsaved={(all, id) => {
      onchange(all);
      onselect(id);
      editing = null;
    }}
    ondeleted={(all) => {
      onchange(all);
      editing = null;
    }}
  />
{/if}

<style>
  .picker {
    position: relative;
  }
  .trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 260px;
    max-width: 100%;
    text-align: left;
    cursor: pointer;
  }
  .trigger :global(svg) {
    width: 16px;
    height: 16px;
    flex: none;
    color: var(--muted);
  }
  .current {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .menu {
    position: absolute;
    left: 0;
    bottom: calc(100% + 6px);
    z-index: 20;
    width: max(100%, 320px);
    max-height: min(420px, 60vh);
    overflow-y: auto;
    padding: 6px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    box-shadow: 0 12px 32px rgb(0 0 0 / 0.16);
  }
  .group {
    padding: 8px 10px 4px;
    color: var(--muted);
    font-size: 0.82em;
  }
  .item {
    display: flex;
    align-items: center;
    border-radius: var(--radius);
  }
  .item:hover {
    background: var(--accent-soft);
  }
  .item.selected {
    background: var(--accent-soft);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .choose {
    flex: 1;
    min-width: 0;
    display: grid;
    gap: 1px;
    padding: 7px 10px;
    border: 0;
    background: transparent;
    text-align: left;
    cursor: pointer;
  }
  .name,
  .summary {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .selected .name {
    font-weight: 600;
  }
  .summary {
    color: var(--muted);
    font-size: 0.86em;
  }
  .item .icon-btn {
    margin-right: 4px;
  }
  .new {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    margin-top: 6px;
    padding: 9px 10px;
    border: 0;
    border-top: 1px solid var(--line);
    border-radius: 0 0 var(--radius) var(--radius);
    background: transparent;
    color: var(--accent);
    font-weight: 500;
    text-align: left;
    cursor: pointer;
  }
  .new:hover {
    background: var(--accent-soft);
  }
  .new :global(svg) {
    width: 16px;
    height: 16px;
  }
</style>
