<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api, AUDIO_EXTENSIONS, type Environment, type Preset } from './lib/api';
  import { errorText, lang, locale, shortcut, t } from './lib/i18n/index.svelte';
  import { queue } from './lib/queue.svelte';
  import { currentPreset, persist, settings } from './lib/settings.svelte';
  import { checkForUpdate, dueForAutoCheck } from './lib/updates.svelte';
  import BottomBar from './lib/components/BottomBar.svelte';
  import DropZone from './lib/components/DropZone.svelte';
  import Icon from './lib/components/Icon.svelte';
  import QueueList from './lib/components/QueueList.svelte';
  import SettingsPanel from './lib/components/SettingsPanel.svelte';
  import Wave from './lib/components/Wave.svelte';

  let env = $state<Environment | null>(null);
  let presets = $state<Preset[]>([]);
  let dragging = $state(false);
  let panelOpen = $state(false);
  let toast = $state<string | null>(null);
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  const ready = $derived(env !== null && !env.ffmpegError);
  const current = $derived(currentPreset(presets));

  /** Fortschritt beim Einlesen, damit große Ordner nicht wie ein Absturz wirken */
  const readingText = $derived.by(() => {
    const p = queue.addProgress;
    if (!p) return '';
    const n = (x: number) => x.toLocaleString(locale());
    const m = t().reading;
    if (p.phase === 'scanning') return p.found > 0 ? m.scanningFound(n(p.found)) : m.scanning;
    return m.progress(n(p.done), n(p.total));
  });

  // Theme, Dichte, Akzent und Sprache auf <html> anwenden und speichern.
  $effect(() => {
    const root = document.documentElement;
    root.dataset.theme = settings.theme;
    root.dataset.density = settings.density;
    root.style.setProperty('--accent', settings.accent);
    root.lang = lang();
    persist();
  });

  // Tray-Symbol passend zur Einstellung; läuft erneut bei Sprachwechsel (Menütexte).
  $effect(() => {
    const enabled = settings.minimizeToTray;
    const labels = { ...t().tray };
    api.setMinimizeToTray(enabled, labels).catch((e) => console.error(e));
  });

  function showToast(text: string) {
    toast = text;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), 4000);
  }

  async function addPaths(paths: string[]) {
    const m = t().toast;
    try {
      const { added, unsupported, skipped } = await queue.add(paths);
      if (added === 0 && unsupported === 0 && skipped === 0) {
        showToast(m.noAudioFiles);
      } else if (added === 0 && unsupported === 0) {
        showToast(m.alreadyListed(skipped));
      } else if (added === 0) {
        showToast(m.noAudioTrack(unsupported));
      } else if (skipped > 0) {
        showToast(m.addedSomeKnown(added, skipped));
      }
    } catch (e) {
      showToast(m.readFailed(errorText(e)));
    }
  }

  async function pickFiles() {
    const selected = await open({
      multiple: true,
      filters: [{ name: t().app.audioFilter, extensions: AUDIO_EXTENSIONS }],
    });
    if (selected) addPaths(Array.isArray(selected) ? selected : [selected]);
  }

  async function pickFolder() {
    const selected = await open({ directory: true, multiple: true });
    if (selected) addPaths(Array.isArray(selected) ? selected : [selected]);
  }

  async function pickOutput() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === 'string') settings.outputDir = selected;
  }

  async function convert() {
    const preset = current;
    if (!preset) return;
    try {
      const started = await api.start({
        ids: queue.startableIds,
        presetId: preset.id,
        outputDir: settings.outputDir,
        template: settings.template,
        overwrite: settings.overwrite,
        subfolders: settings.subfolders,
        lang: lang(),
        fallbacks: t().fallbacks,
      });
      if (started === 0) showToast(t().toast.nothingToDo);
    } catch (e) {
      showToast(errorText(e));
    }
  }

  function onKeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;
    if (mod && e.key.toLowerCase() === 'o') {
      e.preventDefault();
      e.shiftKey ? pickFolder() : pickFiles();
    } else if (mod && e.key === ',') {
      e.preventDefault();
      panelOpen = !panelOpen;
    } else if (e.key === 'Escape' && panelOpen && !document.querySelector('[role="dialog"]')) {
      // Ein offener Dialog schließt sich mit Esc selbst; das Panel bleibt dann offen.
      panelOpen = false;
    }
  }

  onMount(() => {
    const cleanups: Array<() => void> = [];

    (async () => {
      const [environment, list] = await Promise.all([api.environment(), api.presets()]);
      env = environment;
      presets = list;

      cleanups.push(await api.onJobUpdate((u) => queue.apply(u)));
      cleanups.push(await api.onAddProgress((p) => queue.applyProgress(p)));
      cleanups.push(await api.onBatchFinished((s) => showToast(t().summary(s))));
      cleanups.push(
        await getCurrentWebview().onDragDropEvent((event) => {
          const p = event.payload;
          if (p.type === 'enter' || p.type === 'over') dragging = true;
          else if (p.type === 'drop') {
            dragging = false;
            addPaths(p.paths);
          } else dragging = false;
        }),
      );

      // Automatische Update-Prüfung (nur aktiv, wenn eine Update-Quelle eingetragen ist)
      if (settings.autoUpdateCheck && dueForAutoCheck()) {
        const status = await checkForUpdate(environment.appVersion);
        if (status.state === 'available') showToast(t().toast.updateAvailable(status.info.version));
      }
    })();

    return () => cleanups.forEach((fn) => fn());
  });
</script>

<svelte:window onkeydown={onKeydown} />

<div class="app" class:panel-open={panelOpen}>
  <header class="top">
    <div class="brand">
      <span class="mark" aria-hidden="true"><i></i><i></i><i></i><i></i></span>
      Audio Konverter
    </div>
    <div class="top-actions">
      {#if queue.adding && queue.items.length > 0}
        <span class="reading" role="status" aria-live="polite" title={readingText}>
          <Wave size="small" motion="wave" />
          <span class="reading-text">{readingText}</span>
        </span>
      {/if}
      {#if queue.items.length > 0}
        <button class="btn quiet" onclick={pickFiles} title={t().app.addFilesTitle(shortcut('mod', 'O'))}>
          <Icon name="plus" /> {t().app.addFiles}
        </button>
        <button class="btn quiet" onclick={pickFolder} title={t().app.addFolderTitle(shortcut('mod', 'shift', 'O'))}>
          <Icon name="folder" /> {t().app.addFolder}
        </button>
      {/if}
      <button
        class="icon-btn"
        class:on={panelOpen}
        aria-label={t().app.settings}
        aria-expanded={panelOpen}
        title={t().app.settingsTitle(shortcut('mod', ','))}
        onclick={() => (panelOpen = !panelOpen)}
      >
        <Icon name="settings" />
      </button>
    </div>
  </header>

  {#if env?.ffmpegError}
    <div class="notice" role="alert">
      <Icon name="alert" />
      <span>{t().app.ffmpegMissing}</span>
    </div>
  {/if}

  <main class="content">
    <section class="work">
      {#if queue.items.length === 0}
        <DropZone
          active={dragging}
          busy={queue.adding}
          status={readingText}
          onpickfiles={pickFiles}
          onpickfolder={pickFolder}
        />
      {:else}
        <QueueList />
      {/if}
    </section>

    {#if panelOpen}
      <button class="scrim" aria-label={t().app.closeSettings} onclick={() => (panelOpen = false)}></button>
      <div class="side">
        <SettingsPanel
          {env}
          {presets}
          onpresetschange={(all) => (presets = all)}
          ontoast={showToast}
          onclose={() => (panelOpen = false)}
        />
      </div>
    {/if}
  </main>

  <BottomBar
    {presets}
    {current}
    {ready}
    onpresetschange={(all) => (presets = all)}
    onconvert={convert}
    oncancelall={() => api.cancelAll()}
    onpickoutput={pickOutput}
  />

  {#if dragging && queue.items.length > 0}
    <div class="drop-hint" aria-hidden="true">{t().app.dropHint}</div>
  {/if}

  {#if toast}
    <div class="toast" role="status">{toast}</div>
  {/if}
</div>

<style>
  .app {
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .app > :global(header),
  .app > :global(footer),
  .notice {
    flex: none;
  }

  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 14px 10px 18px;
  }
  .brand {
    flex: none;
    display: flex;
    align-items: center;
    gap: 10px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }
  .mark {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    height: 16px;
  }
  .mark i {
    width: 3px;
    border-radius: 2px;
    background: var(--accent);
  }
  .mark i:nth-child(1) { height: 45%; }
  .mark i:nth-child(2) { height: 100%; }
  .mark i:nth-child(3) { height: 70%; }
  .mark i:nth-child(4) { height: 35%; }

  .top-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    min-width: 0;
  }
  .top-actions :global(.btn svg) {
    width: 16px;
    height: 16px;
  }
  .reading {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    margin-right: 8px;
    color: var(--muted);
    font-size: 0.92em;
    font-variant-numeric: tabular-nums;
  }
  .reading-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .icon-btn.on {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .notice {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    margin: 0 18px 10px;
    padding: 10px 12px;
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--danger) 12%, transparent);
    color: var(--danger);
  }
  .notice :global(svg) {
    width: 18px;
    height: 18px;
    flex: none;
  }

  /* Mittelteil: nimmt den restlichen Platz ein, scrollt aber nie selbst */
  .content {
    position: relative;
    flex: 1;
    min-height: 0;
    display: flex;
    overflow: hidden;
  }
  .work {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    padding: 4px 18px 16px;
  }
  .work > :global(*) {
    flex: 1;
    min-height: 0;
  }
  .side {
    flex: none;
    width: 320px;
    min-height: 0;
  }
  .scrim {
    display: none;
  }

  /* Schmales Fenster: Einstellungen legen sich über die Liste */
  @media (max-width: 760px) {
    .side {
      position: absolute;
      top: 0;
      right: 0;
      bottom: 0;
      width: min(340px, 92%);
      z-index: 3;
      box-shadow: -12px 0 32px rgb(0 0 0 / 0.18);
    }
    .scrim {
      display: block;
      position: absolute;
      inset: 0;
      z-index: 2;
      border: 0;
      background: rgb(0 0 0 / 0.25);
    }
  }

  .drop-hint {
    position: absolute;
    inset: 8px;
    z-index: 5;
    display: grid;
    place-items: center;
    border: 2px dashed var(--accent);
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--bg) 82%, transparent);
    color: var(--accent);
    font-size: 1.2em;
    font-weight: 600;
    pointer-events: none;
  }

  .toast {
    position: absolute;
    left: 50%;
    bottom: 86px;
    z-index: 6;
    transform: translateX(-50%);
    padding: 9px 16px;
    border-radius: 999px;
    background: var(--ink);
    color: var(--bg);
    font-weight: 500;
    box-shadow: 0 6px 24px rgb(0 0 0 / 0.2);
    animation: rise 0.18s ease-out;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translate(-50%, 6px);
    }
  }
</style>
