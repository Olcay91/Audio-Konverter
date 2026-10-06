<script lang="ts">
  import { t } from '../i18n/index.svelte';
  import Icon from './Icon.svelte';
  import Wave from './Wave.svelte';

  let {
    active,
    busy,
    status = '',
    onpickfiles,
    onpickfolder,
  }: {
    active: boolean;
    busy: boolean;
    /** Fortschritt beim Einlesen, z. B. „350 von 1.234 Dateien gelesen“ */
    status?: string;
    onpickfiles: () => void;
    onpickfolder: () => void;
  } = $props();
</script>

<div class="zone" class:active>
  <div class="bars">
    <Wave motion={busy ? 'wave' : active ? 'pulse' : 'none'} />
  </div>

  <h1>{busy ? t().dropZone.reading : t().dropZone.title}</h1>
  <p class:status={busy} aria-live="polite">{busy ? status : t().dropZone.text}</p>

  <div class="actions">
    <button class="btn primary" onclick={onpickfiles} disabled={busy}>
      <Icon name="plus" /> {t().dropZone.pickFiles}
    </button>
    <button class="btn" onclick={onpickfolder} disabled={busy}>
      <Icon name="folder" /> {t().dropZone.pickFolder}
    </button>
  </div>
</div>

<style>
  .zone {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 32px;
    text-align: center;
    border: 1.5px dashed var(--line);
    border-radius: var(--radius-lg);
    transition: border-color 0.15s, background 0.15s;
  }
  .zone.active {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .bars {
    display: flex;
    margin-bottom: 14px;
  }
  h1 {
    margin: 0;
    font-size: 1.45em;
    font-weight: 600;
    letter-spacing: -0.01em;
  }
  p {
    margin: 0 0 12px;
    max-width: 44ch;
    min-height: 1.45em;
    color: var(--muted);
  }
  p.status {
    font-variant-numeric: tabular-nums;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
  }
  .actions :global(svg) {
    width: 16px;
    height: 16px;
  }
</style>
