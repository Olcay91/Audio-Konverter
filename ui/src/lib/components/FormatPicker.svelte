<script lang="ts">
  // Format-Auswahl im selben Stil wie der PresetPicker (Menü öffnet nach oben).
  // Die Menü-Stile entsprechen bewusst denen in PresetPicker.svelte.
  import { FORMATS, LOSSLESS, type Codec } from '../api';
  import { t } from '../i18n/index.svelte';
  import Icon from './Icon.svelte';

  let { value, onselect }: { value: Codec; onselect: (codec: Codec) => void } = $props();

  let open = $state(false);
  let root = $state<HTMLDivElement>();

  const lossy = FORMATS.filter((f) => !LOSSLESS.includes(f.codec));
  const lossless = FORMATS.filter((f) => LOSSLESS.includes(f.codec));
  const current = $derived(FORMATS.find((f) => f.codec === value) ?? FORMATS[0]);

  function choose(codec: Codec) {
    onselect(codec);
    open = false;
  }

  // Wie bei einer normalen Auswahlliste: Pfeiltasten wechseln das Format auch bei geschlossenem Menü.
  function onTriggerKeydown(e: KeyboardEvent) {
    if (e.key !== 'ArrowUp' && e.key !== 'ArrowDown') return;
    e.preventDefault();
    const index = FORMATS.findIndex((f) => f.codec === value);
    const next = FORMATS[index + (e.key === 'ArrowDown' ? 1 : -1)];
    if (next) onselect(next.codec);
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
    aria-label={t().bar.format}
    title={t().formats[current.codec]}
    onclick={() => (open = !open)}
    onkeydown={onTriggerKeydown}
  >
    <span class="current">{current.label}</span>
    <Icon name="chevron" />
  </button>

  {#if open}
    <div class="menu" role="listbox" aria-label={t().bar.format}>
      <div class="group">{t().bar.lossy}</div>
      {#each lossy as f (f.codec)}{@render row(f.codec, f.label)}{/each}
      <div class="group">{t().bar.lossless}</div>
      {#each lossless as f (f.codec)}{@render row(f.codec, f.label)}{/each}
    </div>
  {/if}
</div>

{#snippet row(codec: Codec, label: string)}
  <div class="item" class:selected={codec === value}>
    <button class="choose" role="option" aria-selected={codec === value} onclick={() => choose(codec)}>
      <span class="name">{label}</span>
      <span class="summary">{t().formats[codec]}</span>
    </button>
  </div>
{/snippet}

<style>
  .picker {
    position: relative;
  }
  .trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 160px;
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
    width: max(100%, 300px);
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
</style>
