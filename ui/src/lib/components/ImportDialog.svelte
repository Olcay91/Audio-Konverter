<script lang="ts">
  import { onMount } from 'svelte';
  import { api, type ImportResult, type Preset } from '../api';
  import { conflictingPresets, type Backup } from '../backup';
  import { date } from '../format';
  import { errorText, t } from '../i18n/index.svelte';
  import { applySettings, type Settings } from '../settings.svelte';
  import Icon from './Icon.svelte';

  let {
    backup,
    presets,
    onpresetschange,
    ontoast,
    onclose,
  }: {
    backup: Backup;
    /** Aktuelle Presets, um gleichnamige zu erkennen */
    presets: Preset[];
    onpresetschange: (all: Preset[]) => void;
    ontoast: (text: string) => void;
    onclose: () => void;
  } = $props();

  // Der Dialog wird für jeden Import neu erzeugt, daher reichen die Startwerte.
  const hasSettings = Object.keys(backup.settings).length > 0;
  const incoming = backup.presets;
  const conflicts = conflictingPresets(incoming, presets).length;

  let useSettings = $state(hasSettings);
  let usePresets = $state(incoming.length > 0);
  /** Gleichnamige Presets: ersetzen oder vorhandene behalten */
  let replace = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let confirmButton = $state<HTMLButtonElement>();

  onMount(() => confirmButton?.focus());

  /** Preset-Auswahl je Format auf die IDs umstellen, die die Presets in der App bekommen haben. */
  function remap(byFormat: Settings['presetByFormat'], idMap: Record<string, string>): Settings['presetByFormat'] {
    const result: Settings['presetByFormat'] = {};
    for (const [codec, id] of Object.entries(byFormat) as [keyof typeof byFormat, string][]) {
      result[codec] = idMap[id] ?? id;
    }
    return result;
  }

  async function confirm() {
    busy = true;
    error = null;
    try {
      let imported: ImportResult | null = null;
      if (usePresets && incoming.length > 0) {
        imported = await api.importPresets(incoming, replace);
        onpresetschange(imported.presets);
      }
      if (useSettings && hasSettings) {
        const values = { ...backup.settings };
        if (values.presetByFormat && imported) values.presetByFormat = remap(values.presetByFormat, imported.idMap);
        applySettings(values);
      }
      // Erst nach dem Übernehmen übersetzen, damit eine importierte Sprache schon gilt.
      ontoast(
        t().importDialog.result({
          settings: useSettings && hasSettings,
          added: imported?.added ?? 0,
          replaced: imported?.replaced ?? 0,
          skipped: imported?.skipped ?? 0,
          invalid: imported?.invalid ?? 0,
        }),
      );
      onclose();
    } catch (e) {
      error = errorText(e);
    } finally {
      busy = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && !busy && onclose()} />

<button class="backdrop" tabindex="-1" aria-label={t().importDialog.cancel} onclick={onclose}></button>

<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="import-title">
  <header>
    <div>
      <h2 id="import-title">{t().importDialog.title}</h2>
      {#if backup.exportedAt}
        <span class="sub">{t().importDialog.exportedAt(date(backup.exportedAt), backup.appVersion)}</span>
      {/if}
    </div>
    <button class="icon-btn" aria-label={t().importDialog.cancel} onclick={onclose}><Icon name="close" /></button>
  </header>

  <div class="body">
    {#if hasSettings}
      <label class="option">
        <input type="checkbox" class="check" bind:checked={useSettings} />
        <span class="text">
          <span>{t().importDialog.settings}</span>
          <small>{t().importDialog.settingsHint}</small>
        </span>
      </label>
    {/if}

    {#if incoming.length > 0}
      <label class="option">
        <input type="checkbox" class="check" bind:checked={usePresets} />
        <span class="text"><span>{t().importDialog.presets(incoming.length)}</span></span>
      </label>

      {#if usePresets && conflicts > 0}
        <div class="conflicts">
          <p>{t().importDialog.conflicts(conflicts)}</p>
          <div class="segmented" role="radiogroup" aria-label={t().importDialog.conflicts(conflicts)}>
            <button type="button" role="radio" aria-checked={!replace} onclick={() => (replace = false)}>
              {t().importDialog.keep}
            </button>
            <button type="button" role="radio" aria-checked={replace} onclick={() => (replace = true)}>
              {t().importDialog.replace}
            </button>
          </div>
        </div>
      {/if}
    {:else}
      <p class="sub">{t().importDialog.noPresets}</p>
    {/if}

    {#if error}<p class="error" role="alert">{error}</p>{/if}
  </div>

  <footer>
    <button type="button" class="btn" onclick={onclose}>{t().importDialog.cancel}</button>
    <button
      type="button"
      class="btn primary"
      bind:this={confirmButton}
      disabled={busy || (!(useSettings && hasSettings) && !(usePresets && incoming.length > 0))}
      onclick={confirm}
    >
      {t().importDialog.confirm}
    </button>
  </footer>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    border: 0;
    cursor: default;
    z-index: 40;
    background: rgb(0 0 0 / 0.32);
  }
  .dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    z-index: 41;
    width: min(440px, calc(100vw - 32px));
    max-height: calc(100vh - 32px);
    overflow-y: auto;
    transform: translate(-50%, -50%);
    padding: 18px 20px 16px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    box-shadow: 0 24px 64px rgb(0 0 0 / 0.28);
  }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 14px;
  }
  h2 {
    margin: 0;
    font-size: 1.15em;
    font-weight: 600;
  }
  .sub {
    color: var(--muted);
    font-size: 0.9em;
  }
  p.sub {
    margin: 0;
  }
  .body {
    display: grid;
    gap: 14px;
  }
  .option {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    cursor: pointer;
  }
  .check {
    width: 16px;
    height: 16px;
    margin: 2px 0 0;
    flex: none;
    accent-color: var(--accent);
    cursor: pointer;
  }
  .text {
    display: grid;
    gap: 2px;
  }
  .text small {
    color: var(--muted);
    font-size: 0.86em;
    line-height: 1.35;
  }
  .conflicts {
    display: grid;
    gap: 8px;
    margin-left: 26px;
    padding: 10px 12px;
    border-radius: var(--radius);
    background: var(--surface-sunk);
  }
  .conflicts p {
    margin: 0;
    font-size: 0.92em;
  }
  .segmented {
    display: inline-flex;
    justify-self: start;
    padding: 2px;
    border-radius: var(--radius);
    background: var(--bg);
  }
  .segmented button {
    height: 28px;
    padding: 0 11px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
  }
  .segmented button[aria-checked='true'] {
    background: var(--surface);
    color: var(--ink);
    font-weight: 500;
    box-shadow: 0 0 0 1px var(--line);
  }
  .error {
    margin: 0;
    color: var(--danger);
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 18px;
  }
</style>
