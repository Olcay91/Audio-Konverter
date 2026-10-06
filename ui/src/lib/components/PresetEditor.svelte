<script lang="ts">
  import { onMount } from 'svelte';
  import {
    api,
    BIT_DEPTHS,
    bitratesFor,
    FORMATS,
    LOSSLESS,
    qualitiesFor,
    sampleRatesFor,
    type Preset,
    type Rate,
  } from '../api';
  import { sampleRate as formatRate } from '../format';
  import { errorText, t } from '../i18n/index.svelte';
  import Dropdown from './Dropdown.svelte';
  import Icon from './Icon.svelte';

  let {
    preset,
    onclose,
    onsaved,
    ondeleted,
  }: {
    preset: Preset;
    onclose: () => void;
    onsaved: (all: Preset[], id: string) => void;
    ondeleted: (all: Preset[]) => void;
  } = $props();

  // Der Editor wird für jedes Bearbeiten neu erzeugt, daher reicht der Startwert.
  const start = $state.snapshot(preset);
  const codec = start.codec;
  const isNew = start.id === '';
  const formatLabel = FORMATS.find((f) => f.codec === codec)?.label ?? codec;
  const lossless = LOSSLESS.includes(codec);
  const qualities = qualitiesFor(codec);
  const bitrates = bitratesFor(codec);
  const rates = sampleRatesFor(codec);

  let name = $state(start.name);
  let mode = $state<'quality' | 'bitrate'>(start.rate.mode === 'quality' ? 'quality' : 'bitrate');
  let quality = $state(start.rate.mode === 'quality' ? start.rate.value : (qualities[0]?.value ?? 0));
  let kbps = $state(
    start.rate.mode === 'bitrate' ? start.rate.kbps : bitrates.includes(192) ? 192 : (bitrates[0] ?? 128),
  );
  let compression = $state(start.rate.mode === 'lossless' ? (start.rate.compression ?? 5) : 5);
  let sampleRate = $state<number | null>(start.sampleRate);
  let bitDepth = $state<number | null>(start.bitDepth);
  let channels = $state<number | null>(start.channels);

  let error = $state<string | null>(null);
  let saving = $state(false);
  let confirmDelete = $state(false);
  let nameInput = $state<HTMLInputElement>();

  // Eine Bitrate aus einem Standard-Preset, die nicht in der Liste steht, trotzdem anbieten.
  const bitrateOptions = bitrates.includes(kbps) ? bitrates : [...bitrates, kbps].sort((a, b) => a - b);
  /** FLAC-Kompressionsstufen 0–12 mit Hinweis an den wichtigsten Stufen */
  const compressionOptions = $derived(
    Array.from({ length: 13 }, (_, level) => {
      const m = t().editor;
      const note = level === 0 ? m.fastest : level === 5 ? m.standard : level === 12 ? m.smallest : '';
      return { value: level, label: note ? `${level} (${note})` : String(level) };
    }),
  );

  onMount(() => nameInput?.focus());

  function build(): Preset {
    let rate: Rate;
    if (lossless) rate = { mode: 'lossless', compression: codec === 'flac' ? compression : null };
    else if (mode === 'quality' && qualities.length > 0) rate = { mode: 'quality', value: quality };
    else rate = { mode: 'bitrate', kbps };
    return {
      id: start.id,
      name: name.trim(),
      description: '',
      codec,
      rate,
      sampleRate: rates.length > 0 ? sampleRate : null,
      bitDepth: lossless ? bitDepth : null,
      channels,
      builtin: false,
    };
  }

  async function save(e: SubmitEvent) {
    e.preventDefault();
    error = null;
    if (!name.trim()) {
      error = t().errors.presetNameMissing('');
      nameInput?.focus();
      return;
    }
    saving = true;
    try {
      const result = await api.savePreset(build());
      onsaved(result.presets, result.id);
    } catch (err) {
      error = errorText(err);
    } finally {
      saving = false;
    }
  }

  async function remove() {
    if (!confirmDelete) {
      confirmDelete = true;
      return;
    }
    try {
      ondeleted(await api.deletePreset(start.id));
    } catch (err) {
      error = errorText(err);
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} />

<button class="backdrop" tabindex="-1" aria-label={t().editor.close} onclick={onclose}></button>

<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="preset-title">
  <header>
    <div>
      <h2 id="preset-title">{isNew ? t().editor.titleNew : t().editor.titleEdit}</h2>
      <span class="format">{t().editor.format(formatLabel)}</span>
    </div>
    <button class="icon-btn" aria-label={t().editor.close} onclick={onclose}><Icon name="close" /></button>
  </header>

  <form onsubmit={save}>
    <label class="row">
      <span>{t().editor.name}</span>
      <input class="input" bind:this={nameInput} bind:value={name} maxlength="60" placeholder={t().editor.namePlaceholder} />
    </label>

    {#if !lossless}
      {#if qualities.length > 0}
        <div class="row">
          <span>{t().editor.mode}</span>
          <div class="segmented" role="radiogroup" aria-label={t().editor.mode}>
            <button type="button" role="radio" aria-checked={mode === 'quality'} onclick={() => (mode = 'quality')}>
              {codec === 'mp3' ? t().editor.vbr : t().editor.qualityLevel}
            </button>
            <button type="button" role="radio" aria-checked={mode === 'bitrate'} onclick={() => (mode = 'bitrate')}>
              {codec === 'mp3' ? t().editor.cbr : t().editor.bitrate}
            </button>
          </div>
        </div>
      {/if}

      {#if mode === 'quality' && qualities.length > 0}
        <div class="row">
          <span>{t().editor.quality}</span>
          <Dropdown
            block
            label={t().editor.quality}
            value={quality}
            options={qualities.map((q) => ({ value: q.value, label: t().editor.qualityOption(q.label, q.kbps) }))}
            onchange={(v) => (quality = v)}
          />
        </div>
      {:else}
        <div class="row">
          <span>{t().editor.bitrate}</span>
          <Dropdown
            block
            label={t().editor.bitrate}
            value={kbps}
            options={bitrateOptions.map((b) => ({ value: b, label: t().units.kbps(String(b)) }))}
            onchange={(v) => (kbps = v)}
          />
        </div>
      {/if}
    {:else if codec === 'flac'}
      <div class="row">
        <span>{t().editor.compression}</span>
        <Dropdown
          block
          label={t().editor.compression}
          value={compression}
          options={compressionOptions}
          onchange={(v) => (compression = v)}
        />
      </div>
    {/if}

    {#if rates.length > 0}
      <div class="row">
        <span>{t().editor.sampleRate}</span>
        <Dropdown
          block
          label={t().editor.sampleRate}
          value={sampleRate}
          options={[
            { value: null, label: t().editor.asSource },
            ...rates.map((r) => ({ value: r, label: t().editor.atMost(formatRate(r)) })),
          ]}
          onchange={(v) => (sampleRate = v)}
        />
      </div>
    {/if}

    {#if lossless}
      <div class="row">
        <span>{t().editor.bitDepth}</span>
        <Dropdown
          block
          label={t().editor.bitDepth}
          value={bitDepth}
          options={[
            { value: null, label: t().editor.asSource },
            ...BIT_DEPTHS.map((d) => ({ value: d, label: t().editor.atMost(t().units.bits(d)) })),
          ]}
          onchange={(v) => (bitDepth = v)}
        />
      </div>
    {/if}

    <div class="row">
      <span>{t().editor.channels}</span>
      <Dropdown
        block
        label={t().editor.channels}
        value={channels}
        options={[
          { value: null, label: t().editor.asSource },
          { value: 2, label: t().editor.atMostStereo },
          { value: 1, label: t().editor.mono },
        ]}
        onchange={(v) => (channels = v)}
      />
    </div>

    <p class="hint">{t().editor.hint}</p>

    {#if error}<p class="error" role="alert">{error}</p>{/if}

    <footer>
      {#if !isNew}
        <button type="button" class="btn danger" onclick={remove}>
          {confirmDelete ? t().editor.confirmDelete : t().editor.delete}
        </button>
      {/if}
      <span class="spacer"></span>
      <button type="button" class="btn" onclick={onclose}>{t().editor.cancel}</button>
      <button type="submit" class="btn primary" disabled={saving}>{t().editor.save}</button>
    </footer>
  </form>
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
  .format {
    color: var(--muted);
    font-size: 0.9em;
  }
  form {
    display: grid;
    gap: 12px;
  }
  .row {
    display: grid;
    grid-template-columns: 96px minmax(0, 1fr);
    align-items: center;
    gap: 12px;
  }
  .row > span {
    color: var(--muted);
  }
  .segmented {
    display: inline-flex;
    justify-self: start;
    padding: 2px;
    border-radius: var(--radius);
    background: var(--surface-sunk);
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
  .hint {
    margin: 0;
    color: var(--muted);
    font-size: 0.86em;
  }
  .error {
    margin: 0;
    color: var(--danger);
  }
  footer {
    display: flex;
    gap: 8px;
    margin-top: 6px;
  }
  .spacer {
    flex: 1;
  }
  @media (max-width: 480px) {
    .row {
      grid-template-columns: 1fr;
      gap: 4px;
    }
  }
</style>
