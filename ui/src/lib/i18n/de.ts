// Deutsche Texte. Diese Datei ist die Referenz: en.ts muss denselben Aufbau haben.
import type { BatchSummary } from '../api';

const files = (n: number) => (n === 1 ? '1 Datei' : `${n} Dateien`);
const presetsN = (n: number) => (n === 1 ? '1 Preset' : `${n} Presets`);

export const de = {
  /** Tastenbezeichnungen außerhalb von macOS (auf dem Mac gelten Symbole, siehe index) */
  keys: { mod: 'Strg', shift: 'Umschalt', del: 'Entf' },

  app: {
    addFiles: 'Dateien',
    addFolder: 'Ordner',
    addFilesTitle: (keys: string) => `Dateien hinzufügen (${keys})`,
    addFolderTitle: (keys: string) => `Ordner hinzufügen (${keys})`,
    settings: 'Einstellungen',
    settingsTitle: (keys: string) => `Einstellungen (${keys})`,
    closeSettings: 'Einstellungen schließen',
    ffmpegMissing:
      'ffmpeg wurde nicht gefunden. Installiere ffmpeg (z. B. per Paketmanager) oder lege ffmpeg und ffprobe neben die App und starte sie neu.',
    dropHint: 'Loslassen, um Dateien hinzuzufügen',
    audioFilter: 'Audiodateien',
  },

  toast: {
    noAudioFiles: 'Keine Audiodateien gefunden.',
    alreadyListed: (n: number): string =>
      n === 1 ? 'Die Datei ist bereits in der Liste.' : 'Diese Dateien sind bereits in der Liste.',
    noAudioTrack: (n: number): string =>
      n === 1 ? 'Die Datei enthält keine Audiospur.' : 'Keine der Dateien enthält eine Audiospur.',
    addedSomeKnown: (added: number, known: number) =>
      `${files(added)} hinzugefügt, ${files(known)} waren schon in der Liste.`,
    readFailed: (error: string) => `Dateien konnten nicht gelesen werden: ${error}`,
    nothingToDo: 'Es gibt gerade nichts zu konvertieren.',
    updateAvailable: (version: string) => `Version ${version} ist verfügbar. Mehr dazu in den Einstellungen.`,
  },

  /** Meldung, wenn ein Stapel fertig ist */
  summary: (s: BatchSummary): string => {
    const extra =
      (s.skipped > 0 ? `, ${s.skipped} übersprungen` : '') + (s.cancelled > 0 ? `, ${s.cancelled} abgebrochen` : '');
    if (s.done === 0 && s.failed === 0 && s.skipped > 0 && s.cancelled === 0) {
      return s.skipped === 1
        ? 'Nichts konvertiert: Die Datei hat bereits das Zielformat. Details stehen in der Liste.'
        : `Nichts konvertiert: ${s.skipped} Dateien haben bereits das Zielformat. Details stehen in der Liste.`;
    }
    if (s.done === 0 && s.failed === 0)
      return `Konvertierung abgebrochen${s.skipped > 0 ? `, ${s.skipped} übersprungen` : ''}`;
    if (s.done === 0) {
      return s.failed === 1
        ? 'Die Datei konnte nicht konvertiert werden. Details stehen in der Liste.'
        : `Keine Datei konvertiert. ${s.failed} fehlgeschlagen, Details stehen in der Liste.`;
    }
    if (s.failed === 0) return `${files(s.done)} konvertiert${extra}`;
    return `${s.done} konvertiert, ${s.failed} fehlgeschlagen${extra}. Details stehen in der Liste.`;
  },

  dropZone: {
    title: 'Audiodateien hierher ziehen',
    reading: 'Dateien werden gelesen …',
    text: 'Einzelne Dateien oder ganze Ordner. MP3, FLAC, WAV, AAC, Opus, Ogg und viele mehr.',
    pickFiles: 'Dateien auswählen',
    pickFolder: 'Ordner auswählen',
  },

  /** Fortschritt beim Hinzufügen; Zahlen kommen bereits formatiert */
  reading: {
    scanning: 'Ordner werden durchsucht …',
    scanningFound: (n: string) => `Ordner werden durchsucht … ${n} Audiodateien gefunden`,
    progress: (done: string, total: string) => `${done} von ${total} Dateien gelesen`,
  },

  bar: {
    format: 'Format',
    lossy: 'Verlustbehaftet',
    lossless: 'Verlustfrei',
    preset: 'Preset',
    saveIn: 'Speichern in',
    sourceFolder: 'Ordner der Originaldatei',
    sourceFolderTitle: 'Gleicher Ordner wie die Originaldatei',
    sourceSubfolderTitle: (folder: string) => `Unterordner „${folder}“ im Ordner der Originaldatei`,
    resetTarget: 'Zielordner zurücksetzen',
    resetTargetTitle: 'Zurück zum Ordner der Originaldatei',
    active: (n: number) => `${n} in Arbeit`,
    cancelAll: 'Alle abbrechen',
    convert: 'Konvertieren',
    convertN: (n: number) => `${n} Dateien konvertieren`,
    blockedNoFfmpeg: 'ffmpeg wurde nicht gefunden. Siehe Hinweis oben.',
    blockedNoPreset: 'Bitte ein Preset wählen.',
    blockedEmpty: 'Zuerst Dateien oder Ordner hinzufügen.',
    blockedAllDone: 'Alle Dateien sind bereits konvertiert. Füge neue Dateien hinzu oder entferne die erledigten.',
    blockedNoAudio: 'Die Liste enthält keine Dateien mit Audiospur.',
    blockedAllSkipped:
      'Mit diesem Preset würde sich keine Datei ändern: Alle haben bereits dieses Format in gleicher oder besserer Qualität. Wähle ein anderes Format oder Preset.',
  },

  /** Kurzbeschreibung der Formate im Format-Dropdown */
  formats: {
    mp3: 'Läuft auf praktisch jedem Gerät',
    aac: 'Klein und gut, ideal für Apple und Smartphones',
    opus: 'Beste Qualität bei kleinen Dateien',
    vorbis: 'Offenes Format, z. B. für Spiele',
    flac: 'Verlustfrei komprimiert, weit verbreitet',
    alac: 'Verlustfrei komprimiert für Apple-Geräte',
    wav: 'Unkomprimiert, maximal kompatibel',
    aiff: 'Unkomprimiert, Standard bei Apple',
  } as Record<string, string>,

  presets: {
    none: 'Kein Preset',
    listLabel: 'Presets',
    builtin: 'Standard',
    own: 'Eigene Presets',
    createNew: 'Neues Preset aus aktueller Auswahl',
    edit: 'Bearbeiten',
    editNamed: (name: string) => `${name} bearbeiten`,
    /** Kurzbeschreibung eines Presets */
    vbr: (v: number) => `VBR V${v}`,
    quality: (v: number) => `Qualität q${v}`,
    lossless: 'Verlustfrei',
    atMost: (value: string) => `max. ${value}`,
    mono: 'Mono',
    stereo: 'Stereo',
    asSource: 'wie Quelle',
    /**
     * Übersetzte Namen und Beschreibungen der Standard-Presets nach ID.
     * Leer: Deutsch steht bereits in presets/default.toml.
     */
    names: {} as Record<string, { name: string; description?: string }>,
  },

  units: {
    kbps: (n: string) => `${n} kbit/s`,
    khz: (n: string) => `${n} kHz`,
    bits: (n: number) => `${n} Bit`,
  },

  editor: {
    titleNew: 'Neues Preset',
    titleEdit: 'Preset bearbeiten',
    format: (label: string) => `Format: ${label}`,
    close: 'Schließen',
    name: 'Name',
    namePlaceholder: 'z. B. Auto, Handy, Archiv',
    mode: 'Modus',
    vbr: 'Variabel (VBR)',
    cbr: 'Konstant (CBR)',
    qualityLevel: 'Qualitätsstufe',
    bitrate: 'Bitrate',
    quality: 'Qualität',
    qualityOption: (label: string, kbps: number) => `${label} (ca. ${kbps} kbit/s)`,
    compression: 'Kompression',
    fastest: 'am schnellsten',
    standard: 'Standard',
    smallest: 'am kleinsten',
    sampleRate: 'Abtastrate',
    bitDepth: 'Bit-Tiefe',
    channels: 'Kanäle',
    asSource: 'Wie Quelle',
    atMost: (value: string) => `Höchstens ${value}`,
    atMostStereo: 'Höchstens Stereo',
    mono: 'Mono',
    hint: 'Werte werden nur verringert. Eine Datei mit 44,1 kHz bleibt bei 44,1 kHz, auch wenn 96 kHz gewählt ist.',
    delete: 'Löschen',
    confirmDelete: 'Wirklich löschen?',
    cancel: 'Abbrechen',
    save: 'Preset speichern',
  },

  queue: {
    status: {
      ready: 'Bereit',
      queued: 'Wartet',
      running: 'Läuft',
      done: 'Fertig',
      error: 'Fehler',
      cancelled: 'Abgebrochen',
      unsupported: 'Kein Audio',
      skipped: 'Übersprungen',
    },
    sort: {
      added: 'Hinzugefügt',
      name: 'Name',
      sampleRate: 'Abtastrate',
      bitsPerSample: 'Bit-Tiefe',
      bitRate: 'Bitrate',
    },
    sortLabel: 'Sortieren:',
    sortAria: 'Sortieren nach',
    ascTitle: 'Aufsteigend, klicken für absteigend',
    descTitle: 'Absteigend, klicken für aufsteigend',
    ascAria: 'Aufsteigend sortiert',
    descAria: 'Absteigend sortiert',
    selectAll: 'Alle auswählen',
    selectAllTitle: (keys: string) => `Alle auswählen (${keys})`,
    selected: (n: number) => `${n} ausgewählt`,
    files,
    doneCount: (n: number) => `, ${n} fertig`,
    clearSelection: 'Auswahl aufheben',
    removeTitle: (keys: string) => `Entfernen (${keys})`,
    removeSelected: (n: number) => (n === 1 ? 'Datei entfernen' : `${n} Dateien entfernen`),
    clearFinished: 'Erledigte entfernen',
    clearAll: 'Liste leeren',
    listAria: 'Dateien',
    selectFile: (name: string) => `${name} auswählen`,
    saved: (path: string) => `Gespeichert: ${path}`,
    cancel: 'Abbrechen',
    remove: 'Entfernen',
  },

  skip: {
    sameLossless: (format: string) => `Ist bereits ${format} mit diesen Einstellungen, es würde sich nichts ändern.`,
    notSmaller: (format: string) =>
      `Ist bereits ${format} mit gleicher oder höherer Bitrate. Erneutes Umwandeln würde die Qualität nur verschlechtern.`,
  },

  /** Fehlertexte nach Code (Rust: Error::code, PresetError::code, UiError in commands.rs) */
  errors: {
    toolMissing: (tool: string) => `${tool || 'ffmpeg'} wurde nicht gefunden oder ist nicht ausführbar.`,
    io: (detail: string) => `Dateizugriff fehlgeschlagen: ${detail}`,
    probeParse: (detail: string) => `ffprobe-Ausgabe ist unlesbar: ${detail}`,
    probeFailed: (detail: string) => `Datei konnte nicht gelesen werden: ${detail}`,
    unknownFormat: () => 'Datei konnte nicht gelesen werden: unbekanntes Format',
    noAudio: () => 'Datei enthält keine Audiospur',
    presetFile: (detail: string) => `Preset-Datei ist ungültig: ${detail}`,
    userPresets: (detail: string) => `Eigene Presets konnten nicht gelesen oder gespeichert werden: ${detail}`,
    ffmpegFailed: (detail: string) => detail,
    ffmpegExit: (code: string) => `ffmpeg wurde mit Code ${code} beendet`,
    cancelled: () => 'Abgebrochen',
    internal: (detail: string) => `Interner Fehler: ${detail}`,
    fileTooLarge: () => 'Die Datei ist zu groß.',
    presetNameMissing: () => 'Bitte einen Namen für das Preset eingeben.',
    presetNameTooLong: () => 'Der Name darf höchstens 60 Zeichen lang sein.',
    presetRateMismatch: () => 'Diese Qualitätseinstellung passt nicht zum gewählten Format.',
    presetSampleRate: () => 'Diese Abtastrate unterstützt das Format nicht.',
    presetBitDepth: () => 'Eine Bit-Tiefe lässt sich nur bei verlustfreien Formaten festlegen.',
    presetChannels: () => 'Kanäle: nur Mono oder Stereo möglich.',
    presetBuiltinReadonly: () => 'Standard-Presets können nicht geändert oder gelöscht werden.',
    presetDuplicateName: () => 'Für dieses Format gibt es schon ein Preset mit diesem Namen.',
    presetStorageMissing: () => 'Kein Speicherort für eigene Presets gefunden.',
    presetMissing: () => 'Das gewählte Preset gibt es nicht mehr.',
    backupInvalidJson: () => 'Die Datei ist keine gültige JSON-Datei.',
    backupNotABackup: () => 'Die Datei ist keine Einstellungsdatei des Audio Konverters.',
    backupNewerVersion: () => 'Die Datei stammt aus einer neueren Version der App. Bitte zuerst die App aktualisieren.',
    unknown: (code: string) => `Unbekannter Fehler (${code})`,
  } as Record<string, (detail: string) => string>,

  /** Menü des Symbols im Infobereich */
  tray: { show: 'Audio Konverter anzeigen', quit: 'Beenden', tooltip: 'Audio Konverter' },

  /** Ersatz für fehlende Tags in der Dateinamen-Vorlage */
  fallbacks: { artist: 'Unbekannter Interpret', album: 'Unbekanntes Album' },

  settings: {
    title: 'Einstellungen',
    close: 'Einstellungen schließen',

    general: 'Allgemein',
    language: 'Sprache',
    languages: { system: 'Systemsprache', de: 'Deutsch', en: 'English' },
    mode: 'Ansicht',
    modes: { simple: 'Einfach', advanced: 'Erweitert' },
    modeHintSimple: 'Zeigt alles für den Alltag, inklusive eigener Presets und Dateinamen.',
    modeHintAdvanced: 'Zeigt zusätzlich „Vorhandene Dateien überschreiben“ und ffmpeg-Details. Diese Bereiche sind grün umrandet.',
    hiddenActive: '„Vorhandene Dateien überschreiben“ ist eingeschaltet und gilt weiter, auch wenn der Schalter hier verborgen ist.',

    appearance: 'Darstellung',
    theme: 'Farbschema',
    themes: { system: 'System', light: 'Hell', dark: 'Dunkel' },
    density: 'Listendichte',
    densities: { comfortable: 'Luftig', compact: 'Kompakt' },
    accent: 'Akzentfarbe',
    minimizeToTray: 'In den Infobereich minimieren',
    minimizeToTrayHint:
      'Beim Minimieren verschwindet das Fenster aus der Taskleiste. Ein Klick auf das Symbol im Infobereich holt es zurück, laufende Konvertierungen gehen weiter.',
    accents: {
      petrol: 'Petrol',
      indigo: 'Indigo',
      moss: 'Moos',
      amber: 'Bernstein',
      raspberry: 'Himbeere',
      graphite: 'Graphit',
    } as Record<string, string>,

    output: 'Ausgabe',
    overwrite: 'Vorhandene Dateien überschreiben',
    overwriteHint: 'Ist der Schalter aus, wird bei gleichem Namen „(2)“ angehängt.',
    subfolders: 'Unterordner anlegen',
    subfoldersHintTarget: 'Die Ordnerstruktur hinzugefügter Ordner wird im Zielordner nachgebildet.',
    subfoldersHintSource: (folder: string) =>
      `Neben jeder Originaldatei entsteht ein Unterordner „${folder}“ für die konvertierten Dateien.`,

    fileNames: 'Dateinamen',
    templateLabel: 'Vorlage für neue Dateien. Mit / entstehen Unterordner.',
    example: (path: string) => `Beispiel: ${path}`,
    resetTemplate: 'Zurücksetzen',
    resetTemplateTitle: (template: string) => `Vorlage auf ${template} zurücksetzen`,
    insertPlaceholder: (token: string) => `${token} an der Cursorposition einfügen`,
    duplicatePlaceholders: (list: string, n: number) =>
      n === 1
        ? `${list} kommt mehrfach vor. Ist das gewollt?`
        : `${list} kommen mehrfach vor. Ist das gewollt?`,
    exampleValues: {
      name: 'Originalname',
      artist: 'Interpret',
      albumartist: 'Albuminterpret',
      album: 'Album',
      track: '01',
      title: 'Titel',
      disc: '1',
      year: '2024',
    } as Record<string, string>,

    backup: 'Sichern & übertragen',
    backupHint: 'Einstellungen und eigene Presets als Datei, z. B. für einen zweiten Rechner. Der Zielordner wird nicht mitgesichert.',
    export: 'Exportieren …',
    import: 'Importieren …',
    backupFileName: 'audio-konverter-einstellungen.json',
    backupFilter: 'Einstellungsdatei',
    exported: 'Einstellungen exportiert.',
    exportFailed: (error: string) => `Export fehlgeschlagen: ${error}`,
    importFailed: (error: string) => `Import fehlgeschlagen: ${error}`,

    about: 'Über & Updates',
    version: (v: string) => `Version ${v}`,
    autoCheck: 'Automatisch nach Updates suchen',
    autoCheckHint: 'Höchstens einmal am Tag beim Start der App.',
    checkNow: 'Nach Updates suchen',
    checking: 'Suche …',
    upToDate: 'Du verwendest die neueste Version.',
    updateAvailable: (v: string) => `Version ${v} ist verfügbar.`,
    download: 'Zur Download-Seite',
    updateNow: 'Jetzt aktualisieren',
    downloadingUpdate: (v: string, percent: string | null) =>
      percent ? `Version ${v} wird heruntergeladen … ${percent}` : `Version ${v} wird heruntergeladen …`,
    installingUpdate: (v: string) => `Version ${v} wird installiert. Die App startet gleich neu.`,
    waitForConversions: 'Erst möglich, wenn die laufenden Konvertierungen fertig sind.',
    manualUpdate:
      'Diese Version (portabel oder als Paket installiert) aktualisiert sich nicht selbst. Lade die neue Version von der Download-Seite.',
    updateError: (error: string) => `Update-Prüfung fehlgeschlagen: ${error}`,
    ffmpegNote: 'Verwendet FFmpeg, lizenziert unter der LGPL.',
    parallel: (n: number) => `Bis zu ${n} Dateien gleichzeitig`,
  },

  importDialog: {
    title: 'Einstellungen importieren',
    exportedAt: (date: string, version: string) => `Exportiert am ${date} mit Version ${version}`,
    settings: 'Einstellungen übernehmen',
    settingsHint: 'Ersetzt Darstellung, Sprache, Ausgabe und Dateinamen. Der Zielordner bleibt unverändert.',
    presets: (n: number) => (n === 1 ? '1 eigenes Preset übernehmen' : `${n} eigene Presets übernehmen`),
    noPresets: 'Die Datei enthält keine eigenen Presets.',
    conflicts: (n: number) =>
      n === 1
        ? '1 Preset heißt genauso wie ein vorhandenes. Was soll damit passieren?'
        : `${n} Presets heißen genauso wie vorhandene. Was soll damit passieren?`,
    replace: 'Ersetzen',
    keep: 'Vorhandene behalten',
    confirm: 'Importieren',
    cancel: 'Abbrechen',
    /** Meldung nach dem Import */
    result: (r: { settings: boolean; added: number; replaced: number; skipped: number; invalid: number }) => {
      const parts: string[] = [];
      if (r.settings) parts.push('Einstellungen übernommen');
      if (r.added) parts.push(`${presetsN(r.added)} hinzugefügt`);
      if (r.replaced) parts.push(`${r.replaced} ersetzt`);
      if (r.skipped) parts.push(`${r.skipped} übersprungen`);
      if (r.invalid) parts.push(`${r.invalid} ungültig`);
      return parts.length ? `Import abgeschlossen: ${parts.join(', ')}.` : 'Es wurde nichts importiert.';
    },
  },
};

export type Messages = typeof de;
