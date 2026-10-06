<script lang="ts">
  import { onMount } from 'svelte';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { api, FORMATS, type Environment, type Preset } from '../api';
  import { createBackup, parseBackup, type Backup } from '../backup';
  import { errorText, t } from '../i18n/index.svelte';
  import {
    ACCENTS,
    DEFAULT_TEMPLATE,
    hiddenSettingsActive,
    settings,
    type Density,
    type Language,
    type Mode,
    type Theme,
  } from '../settings.svelte';
  import { queue } from '../queue.svelte';
  import { checkForUpdate, installUpdate, RELEASES_URL, update } from '../updates.svelte';
  import Dropdown from './Dropdown.svelte';
  import Icon from './Icon.svelte';
  import ImportDialog from './ImportDialog.svelte';

  let {
    env,
    presets,
    onpresetschange,
    ontoast,
    onclose,
  }: {
    env: Environment | null;
    presets: Preset[];
    onpresetschange: (all: Preset[]) => void;
    ontoast: (text: string) => void;
    onclose: () => void;
  } = $props();

  const THEMES: Theme[] = ['system', 'light', 'dark'];
  const DENSITIES: Density[] = ['comfortable', 'compact'];
  const LANGUAGES: Language[] = ['system', 'de', 'en'];
  const MODES: Mode[] = ['simple', 'advanced'];
  const PLACEHOLDERS = ['name', 'artist', 'albumartist', 'album', 'track', 'title', 'disc', 'year'];

  const advanced = $derived(settings.mode === 'advanced');
  const folderName = $derived(FORMATS.find((f) => f.codec === settings.format)?.folder ?? 'MP3');

  let templateInput = $state<HTMLInputElement>();
  /** Eingelesene Einstellungsdatei, solange der Import-Dialog offen ist */
  let pendingImport = $state<Backup | null>(null);

  function insert(key: string) {
    if (!templateInput) return;
    const input = templateInput;
    const token = `{${key}}`;
    const start = input.selectionStart ?? settings.template.length;
    const end = input.selectionEnd ?? start;
    settings.template = settings.template.slice(0, start) + token + settings.template.slice(end);
    queueMicrotask(() => {
      input.focus();
      input.setSelectionRange(start + token.length, start + token.length);
    });
  }

  const example = $derived.by(() => {
    const values = t().settings.exampleValues;
    return (settings.template || '{name}').replace(/\{(\w+)\}/g, (all, key: string) => values[key] ?? all);
  });

  /** Weicht die Vorlage vom Standard ab? Dann gibt es „Zurücksetzen“. */
  const templateChanged = $derived(settings.template.trim() !== '' && settings.template.trim() !== DEFAULT_TEMPLATE);

  /** Platzhalter, die mehrfach vorkommen, z. B. versehentlich „{name}{name}“ */
  const duplicates = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const match of settings.template.matchAll(/\{(\w+)\}/g)) {
      const key = match[1].toLowerCase();
      if (PLACEHOLDERS.includes(key)) counts.set(key, (counts.get(key) ?? 0) + 1);
    }
    return [...counts].filter(([, n]) => n > 1).map(([key]) => `{${key}}`);
  });

  function resetTemplate() {
    settings.template = DEFAULT_TEMPLATE;
    templateInput?.focus();
  }

  /** Dateiendung des gewählten Formats für das Beispiel */
  const exampleExt = $derived(
    ({ mp3: 'mp3', aac: 'm4a', alac: 'm4a', opus: 'opus', vorbis: 'ogg', flac: 'flac', wav: 'wav', aiff: 'aiff' })[
      settings.format
    ],
  );

  // ---------- Export / Import ----------

  const fileFilter = () => [{ name: t().settings.backupFilter, extensions: ['json'] }];

  async function exportBackup() {
    const path = await save({ defaultPath: t().settings.backupFileName, filters: fileFilter() });
    if (!path) return;
    try {
      await api.writeTextFile(path, createBackup(settings, presets, env?.appVersion ?? '?'));
      ontoast(t().settings.exported);
    } catch (e) {
      ontoast(t().settings.exportFailed(errorText(e)));
    }
  }

  async function importBackup() {
    const path = await open({ multiple: false, directory: false, filters: fileFilter() });
    if (typeof path !== 'string') return;
    try {
      pendingImport = parseBackup(await api.readTextFile(path));
    } catch (e) {
      ontoast(t().settings.importFailed(errorText(e)));
    }
  }

  // ---------- Updates ----------

  /** Kann sich diese Installation selbst ersetzen? Sonst Link zur Download-Seite. */
  let canSelfUpdate = $state(false);
  onMount(async () => {
    try {
      canSelfUpdate = (await api.installInfo()).canSelfUpdate;
    } catch {
      canSelfUpdate = false;
    }
  });

  /** Während Konvertierungen laufen, nicht installieren: Die App startet dabei neu. */
  const conversionsRunning = $derived(queue.counts.active > 0);

  const updateText = $derived.by(() => {
    const s = update.status;
    const m = t().settings;
    if (s.state === 'current') return m.upToDate;
    if (s.state === 'available') return m.updateAvailable(s.version);
    if (s.state === 'downloading') {
      const percent = s.progress === null ? null : `${Math.round(s.progress * 100)} %`;
      return m.downloadingUpdate(s.version, percent);
    }
    if (s.state === 'installing') return m.installingUpdate(s.version);
    if (s.state === 'error') return m.updateError(s.message);
    return '';
  });

  const updateBusy = $derived(['checking', 'downloading', 'installing'].includes(update.status.state));
</script>

<aside class="panel" aria-label={t().settings.title}>
  <header>
    <h2>{t().settings.title}</h2>
    <button class="icon-btn" aria-label={t().settings.close} onclick={onclose}><Icon name="close" /></button>
  </header>

  <section>
    <h3>{t().settings.about}</h3>
    <p class="about-line">
      <strong>Audio Konverter</strong>
      {#if env}<span class="muted">{t().settings.version(env.appVersion)}</span>{/if}
    </p>

    <label class="toggle">
      <span class="text">
        <span>{t().settings.autoCheck}</span>
        <small>{t().settings.autoCheckHint}</small>
      </span>
      <input type="checkbox" role="switch" bind:checked={settings.autoUpdateCheck} />
    </label>

    <div class="buttons">
      {#if update.status.state === 'available' && canSelfUpdate}
        <!-- Gesperrte Buttons zeigen nicht überall einen Tooltip, daher am Wrapper -->
        <span class="btn-wrap" title={conversionsRunning ? t().settings.waitForConversions : ''}>
          <button class="btn primary" disabled={conversionsRunning} onclick={installUpdate}>
            {t().settings.updateNow}
          </button>
        </span>
      {:else if update.status.state === 'available'}
        <button class="btn primary" onclick={() => openUrl(RELEASES_URL)}>{t().settings.download}</button>
      {:else}
        <button class="btn" disabled={updateBusy} onclick={checkForUpdate}>
          {update.status.state === 'checking' ? t().settings.checking : t().settings.checkNow}
        </button>
      {/if}
    </div>

    {#if update.status.state === 'downloading'}
      <div
        class="update-progress"
        role="progressbar"
        aria-valuemin="0"
        aria-valuemax="100"
        aria-valuenow={update.status.progress === null ? undefined : Math.round(update.status.progress * 100)}
      >
        <span
          class:indeterminate={update.status.progress === null}
          style:width={update.status.progress === null ? '30%' : `${update.status.progress * 100}%`}
        ></span>
      </div>
    {/if}
    {#if updateText}
      <p class="hint" class:error={update.status.state === 'error'} aria-live="polite">{updateText}</p>
    {/if}
    {#if update.status.state === 'available' && !canSelfUpdate}
      <p class="hint">{t().settings.manualUpdate}</p>
    {/if}
  </section>

  <section>
    <h3>{t().settings.general}</h3>

    <div class="setting">
      <span>{t().settings.language}</span>
      <Dropdown
        size="sm"
        label={t().settings.language}
        value={settings.language}
        options={LANGUAGES.map((l) => ({ value: l, label: t().settings.languages[l] }))}
        onchange={(l) => (settings.language = l)}
      />
    </div>

    <div class="setting">
      <span>{t().settings.mode}</span>
      <div class="segmented" role="radiogroup" aria-label={t().settings.mode}>
        {#each MODES as m}
          <button role="radio" aria-checked={settings.mode === m} onclick={() => (settings.mode = m)}>
            {t().settings.modes[m]}
          </button>
        {/each}
      </div>
    </div>
    <p class="hint tight">{advanced ? t().settings.modeHintAdvanced : t().settings.modeHintSimple}</p>
    {#if !advanced && hiddenSettingsActive()}
      <p class="hint notice-line"><Icon name="alert" /> {t().settings.hiddenActive}</p>
    {/if}
  </section>

  <section>
    <h3>{t().settings.appearance}</h3>

    <div class="setting">
      <span>{t().settings.theme}</span>
      <div class="segmented" role="radiogroup" aria-label={t().settings.theme}>
        {#each THEMES as theme}
          <button role="radio" aria-checked={settings.theme === theme} onclick={() => (settings.theme = theme)}>
            {t().settings.themes[theme]}
          </button>
        {/each}
      </div>
    </div>

    <div class="setting">
      <span>{t().settings.density}</span>
      <div class="segmented" role="radiogroup" aria-label={t().settings.density}>
        {#each DENSITIES as d}
          <button role="radio" aria-checked={settings.density === d} onclick={() => (settings.density = d)}>
            {t().settings.densities[d]}
          </button>
        {/each}
      </div>
    </div>

    <div class="setting">
      <span>{t().settings.accent}</span>
      <div class="swatches" role="radiogroup" aria-label={t().settings.accent}>
        {#each ACCENTS as a}
          {@const label = t().settings.accents[a.id] ?? a.id}
          <button
            class="swatch"
            role="radio"
            aria-checked={settings.accent === a.value}
            aria-label={label}
            title={label}
            style:--swatch={a.value}
            onclick={() => (settings.accent = a.value)}
          ></button>
        {/each}
      </div>
    </div>

    <label class="toggle">
      <span class="text">
        <span>{t().settings.minimizeToTray}</span>
        <small>{t().settings.minimizeToTrayHint}</small>
      </span>
      <input type="checkbox" role="switch" bind:checked={settings.minimizeToTray} />
    </label>
  </section>

  <section>
    <h3>{t().settings.output}</h3>

    {#if advanced}
      <div class="adv" data-label={t().settings.modes.advanced}>
        <label class="toggle">
          <span class="text">
            <span>{t().settings.overwrite}</span>
            <small>{t().settings.overwriteHint}</small>
          </span>
          <input type="checkbox" role="switch" bind:checked={settings.overwrite} />
        </label>
      </div>
    {/if}

    <label class="toggle">
      <span class="text">
        <span>{t().settings.subfolders}</span>
        <small>
          {settings.outputDir ? t().settings.subfoldersHintTarget : t().settings.subfoldersHintSource(folderName)}
        </small>
      </span>
      <input type="checkbox" role="switch" bind:checked={settings.subfolders} />
    </label>
  </section>

  <section>
    <h3>{t().settings.fileNames}</h3>
    <div class="setting stacked">
      <div class="template-head">
        <label for="template-input">{t().settings.templateLabel}</label>
        {#if templateChanged}
          <button class="link" title={t().settings.resetTemplateTitle(DEFAULT_TEMPLATE)} onclick={resetTemplate}>
            {t().settings.resetTemplate}
          </button>
        {/if}
      </div>
      <input
        id="template-input"
        class="input"
        bind:this={templateInput}
        bind:value={settings.template}
        spellcheck="false"
        placeholder={DEFAULT_TEMPLATE}
      />
    </div>
    <div class="chips">
      {#each PLACEHOLDERS as p}
        <button class="chip" title={t().settings.insertPlaceholder(`{${p}}`)} onclick={() => insert(p)}>{p}</button>
      {/each}
    </div>
    {#if duplicates.length > 0}
      <p class="hint notice-line warn" role="status">
        <Icon name="alert" />
        {t().settings.duplicatePlaceholders(duplicates.join(', '), duplicates.length)}
      </p>
    {/if}
    <p class="hint">{t().settings.example(`${example}.${exampleExt}`)}</p>
  </section>

  <section>
    <h3>{t().settings.backup}</h3>
    <p class="hint tight">{t().settings.backupHint}</p>
    <div class="buttons">
      <button class="btn" onclick={exportBackup}>{t().settings.export}</button>
      <button class="btn" onclick={importBackup}>{t().settings.import}</button>
    </div>
  </section>

  <section class="ffmpeg">
    {#if env?.ffmpegError}
      <p class="hint error">{errorText(env.ffmpegError)}</p>
    {:else if env}
      <p class="hint">{env.ffmpegVersion}</p>
      {#if advanced}
        <div class="adv compact" data-label={t().settings.modes.advanced}>
          <p class="hint path">{env.ffmpegPath}</p>
          <p class="hint path">{t().settings.parallel(env.parallelJobs)}</p>
        </div>
      {/if}
    {/if}
    <p class="hint">
      {t().settings.ffmpegNote}
      <button class="link" onclick={() => openUrl('https://ffmpeg.org')}>ffmpeg.org</button>
    </p>
  </section>
</aside>

{#if pendingImport}
  <ImportDialog
    backup={pendingImport}
    {presets}
    {onpresetschange}
    {ontoast}
    onclose={() => (pendingImport = null)}
  />
{/if}

<style>
  .panel {
    height: 100%;
    overflow-y: auto;
    padding: 4px 18px 18px;
    background: var(--surface);
    border-left: 1px solid var(--line);
  }
  header {
    position: sticky;
    top: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 0;
    background: var(--surface);
  }
  h2 {
    margin: 0;
    font-size: 1.1em;
    font-weight: 600;
  }
  section {
    padding: 14px 0;
    border-top: 1px solid var(--line);
  }
  h3 {
    margin: 0 0 10px;
    font-size: 0.95em;
    font-weight: 600;
  }
  .setting {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
    color: var(--muted);
  }
  .setting.stacked {
    flex-direction: column;
    align-items: stretch;
    gap: 6px;
  }
  .segmented {
    display: inline-flex;
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
  .swatches {
    display: flex;
    gap: 8px;
  }
  .swatch {
    width: 22px;
    height: 22px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: var(--swatch);
    cursor: pointer;
  }
  .swatch[aria-checked='true'] {
    box-shadow: 0 0 0 2px var(--surface), 0 0 0 4px var(--swatch);
  }
  .toggle {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 14px;
    margin-bottom: 14px;
    cursor: pointer;
  }
  .toggle.disabled {
    cursor: default;
    opacity: 0.6;
  }
  .toggle .text {
    display: grid;
    gap: 2px;
  }
  .toggle small {
    color: var(--muted);
    font-size: 0.86em;
    line-height: 1.35;
  }
  .toggle input {
    appearance: none;
    -webkit-appearance: none;
    flex: none;
    position: relative;
    width: 36px;
    height: 20px;
    margin: 1px 0 0;
    border-radius: 999px;
    background: var(--line);
    cursor: inherit;
    transition: background 0.15s;
  }
  .toggle input::before {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 2px rgb(0 0 0 / 0.25);
    transition: transform 0.15s;
  }
  .toggle input:checked {
    background: var(--accent);
  }
  .toggle input:checked::before {
    transform: translateX(16px);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    padding: 3px 9px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: transparent;
    color: var(--muted);
    font-size: 0.88em;
    cursor: pointer;
  }
  .chip:hover {
    border-color: var(--accent);
    color: var(--accent);
  }
  .hint {
    margin: 10px 0 0;
    color: var(--muted);
    font-size: 0.88em;
    overflow-wrap: anywhere;
    user-select: text;
  }
  .hint.path {
    margin-top: 4px;
  }
  .hint.error {
    color: var(--danger);
  }
  .hint.tight {
    margin: 0 0 12px;
  }
  .notice-line {
    display: flex;
    gap: 6px;
    align-items: flex-start;
    margin: 0;
    color: var(--ink);
  }
  .notice-line :global(svg) {
    width: 15px;
    height: 15px;
    flex: none;
    margin-top: 1px;
    color: var(--accent);
  }
  /* Nur im erweiterten Modus sichtbar: dezenter grüner Rahmen mit kleinem Etikett */
  .adv {
    position: relative;
    margin: 4px 0 14px;
    padding: 12px 12px 10px;
    border: 1px solid var(--ok);
    border: 1px solid color-mix(in srgb, var(--ok) 45%, transparent);
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--ok) 5%, transparent);
    animation: adv-in 0.18s ease-out;
  }
  .adv::after {
    content: attr(data-label);
    position: absolute;
    top: -0.65em;
    right: 10px;
    padding: 0 5px;
    font-size: 0.72em;
    font-weight: 600;
    line-height: 1.2;
    letter-spacing: 0.02em;
    color: var(--ok);
    background: var(--surface);
  }
  .adv.compact {
    margin: 8px 0 4px;
    padding: 8px 10px;
  }
  .adv > :global(:last-child),
  .adv .toggle:last-child {
    margin-bottom: 0;
  }
  .adv .path:first-child {
    margin-top: 0;
  }
  @keyframes adv-in {
    from {
      opacity: 0;
      transform: translateY(-3px);
    }
  }
  .template-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 10px;
  }
  .template-head .link {
    flex: none;
    font-size: 0.92em;
  }
  .notice-line.warn {
    margin-top: 10px;
  }
  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .buttons .btn {
    height: 30px;
    padding: 0 12px;
  }
  .btn-wrap {
    display: inline-flex;
  }
  /* Mausereignisse gehen an den Wrapper, damit dessen Tooltip erscheint */
  .btn-wrap .btn:disabled {
    pointer-events: none;
  }
  .update-progress {
    height: 6px;
    margin-top: 12px;
    overflow: hidden;
    border-radius: 3px;
    background: var(--surface-sunk);
  }
  .update-progress span {
    display: block;
    height: 100%;
    border-radius: 3px;
    background: var(--accent);
    transition: width 0.2s ease-out;
  }
  .update-progress .indeterminate {
    animation: slide 1.2s ease-in-out infinite;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(340%);
    }
  }
  .about-line {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin: 0 0 12px;
  }
  .muted {
    color: var(--muted);
    font-size: 0.92em;
  }
  .ffmpeg {
    padding-bottom: 4px;
  }
  .ffmpeg .hint:first-child {
    margin-top: 0;
  }
  .link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--accent);
    font-size: inherit;
    cursor: pointer;
  }
  .link:hover {
    text-decoration: underline;
  }
</style>
