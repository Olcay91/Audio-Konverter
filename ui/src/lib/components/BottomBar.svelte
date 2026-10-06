<script lang="ts">
  import { FORMATS, type Preset } from '../api';
  import { shortPath } from '../format';
  import { t } from '../i18n/index.svelte';
  import { skipReason } from '../skip';
  import { queue } from '../queue.svelte';
  import { settings } from '../settings.svelte';
  import FormatPicker from './FormatPicker.svelte';
  import Icon from './Icon.svelte';
  import PresetPicker from './PresetPicker.svelte';

  let {
    presets,
    current,
    ready,
    onconvert,
    oncancelall,
    onpickoutput,
    onpresetschange,
  }: {
    presets: Preset[];
    current: Preset | undefined;
    ready: boolean;
    onconvert: () => void;
    oncancelall: () => void;
    onpickoutput: () => void;
    onpresetschange: (all: Preset[]) => void;
  } = $props();

  const formatPresets = $derived(presets.filter((p) => p.codec === settings.format));
  const folderName = $derived(FORMATS.find((f) => f.codec === settings.format)?.folder ?? '');
  // Der Unterordner (z. B. „FLAC“) steht nur im Tooltip, damit die Leiste kurz bleibt.
  const targetLabel = $derived(settings.outputDir ? shortPath(settings.outputDir) : t().bar.sourceFolder);
  const startable = $derived(queue.startableItems);
  /** Dateien, bei denen das gewählte Preset tatsächlich etwas ändert */
  const toConvert = $derived.by(() => {
    const preset = current;
    if (!preset) return startable.length;
    return startable.filter((i) => i.info && !skipReason(preset, i.info)).length;
  });

  /** Warum der Konvertieren-Button gesperrt ist (leer = nicht gesperrt). */
  const blockedReason = $derived.by(() => {
    const m = t().bar;
    if (!ready) return m.blockedNoFfmpeg;
    if (!current) return m.blockedNoPreset;
    if (queue.items.length === 0) return m.blockedEmpty;
    if (startable.length === 0) {
      return queue.items.some((i) => i.status === 'done') ? m.blockedAllDone : m.blockedNoAudio;
    }
    if (toConvert === 0) return m.blockedAllSkipped;
    return '';
  });
</script>

<footer class="bar">
  <div class="field">
    <span class="label">{t().bar.format}</span>
    <FormatPicker value={settings.format} onselect={(codec) => (settings.format = codec)} />
  </div>

  <div class="field">
    <span class="label">{t().bar.preset}</span>
    <PresetPicker
      presets={formatPresets}
      {current}
      onselect={(id) => (settings.presetByFormat[settings.format] = id)}
      onchange={onpresetschange}
    />
  </div>

  <div class="field">
    <span class="label">{t().bar.saveIn}</span>
    <div class="target">
      <button
        class="btn"
        onclick={onpickoutput}
        title={settings.outputDir ??
          (settings.subfolders ? t().bar.sourceSubfolderTitle(folderName) : t().bar.sourceFolderTitle)}
      >
        <Icon name="folder" />
        <span class="path">{targetLabel}</span>
      </button>
      {#if settings.outputDir}
        <button
          class="icon-btn"
          title={t().bar.resetTargetTitle}
          aria-label={t().bar.resetTarget}
          onclick={() => (settings.outputDir = null)}
        >
          <Icon name="close" />
        </button>
      {/if}
    </div>
  </div>

  <div class="go">
    {#if queue.busy}
      <span class="progress-note">{t().bar.active(queue.counts.active)}</span>
      <button class="btn danger" onclick={oncancelall}><Icon name="stop" /> {t().bar.cancelAll}</button>
    {:else}
      <!-- Gesperrte Buttons zeigen nicht überall einen Tooltip, daher am Wrapper -->
      <span class="convert-wrap" title={blockedReason}>
        <button class="btn primary convert" onclick={onconvert} disabled={blockedReason !== ''}>
          {toConvert > 1 ? t().bar.convertN(toConvert) : t().bar.convert}
        </button>
      </span>
    {/if}
  </div>
</footer>

<style>
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 12px 18px;
    padding: 12px 18px 14px;
    background: var(--surface);
    border-top: 1px solid var(--line);
  }
  .field {
    display: grid;
    gap: 4px;
    min-width: 0;
  }
  .label {
    font-size: 0.82em;
    color: var(--muted);
  }
  .target {
    display: flex;
    align-items: center;
    gap: 2px;
    min-width: 0;
  }
  .target .btn {
    max-width: 260px;
    font-weight: 400;
  }
  .path {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .bar :global(.btn svg) {
    width: 16px;
    height: 16px;
    flex: none;
  }
  .go {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .progress-note {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .convert-wrap {
    display: inline-flex;
  }
  /* Mausereignisse gehen an den Wrapper, damit dessen Tooltip erscheint */
  .convert:disabled {
    pointer-events: none;
  }
  .convert {
    height: 38px;
    padding: 0 22px;
    font-size: 1.04em;
  }

  @media (max-width: 620px) {
    .field,
    .go,
    .convert-wrap,
    .convert {
      width: 100%;
    }
    .field :global(.trigger) {
      width: 100%;
    }
    .target .btn {
      flex: 1;
      max-width: none;
    }
    .convert {
      justify-content: center;
    }
  }
</style>
