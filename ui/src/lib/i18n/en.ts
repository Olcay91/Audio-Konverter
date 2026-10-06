// Englische Texte. Aufbau wie de.ts; TypeScript meldet fehlende oder überzählige Einträge.
import type { Messages } from './de';

const files = (n: number) => (n === 1 ? '1 file' : `${n} files`);
const presetsN = (n: number) => (n === 1 ? '1 preset' : `${n} presets`);

export const en: Messages = {
  keys: { mod: 'Ctrl', shift: 'Shift', del: 'Del' },

  app: {
    addFiles: 'Files',
    addFolder: 'Folder',
    addFilesTitle: (keys) => `Add files (${keys})`,
    addFolderTitle: (keys) => `Add folder (${keys})`,
    settings: 'Settings',
    settingsTitle: (keys) => `Settings (${keys})`,
    closeSettings: 'Close settings',
    ffmpegMissing:
      'ffmpeg was not found. Install ffmpeg (e.g. with a package manager) or place ffmpeg and ffprobe next to the app and restart it.',
    dropHint: 'Drop to add files',
    audioFilter: 'Audio files',
  },

  toast: {
    noAudioFiles: 'No audio files found.',
    alreadyListed: (n) => (n === 1 ? 'This file is already in the list.' : 'These files are already in the list.'),
    noAudioTrack: (n) => (n === 1 ? 'The file has no audio track.' : 'None of the files has an audio track.'),
    addedSomeKnown: (added, known) => `${files(added)} added, ${files(known)} already in the list.`,
    readFailed: (error) => `Could not read files: ${error}`,
    nothingToDo: 'There is nothing to convert right now.',
    updateAvailable: (version) => `Version ${version} is available. See settings for details.`,
  },

  summary: (s) => {
    const extra = (s.skipped > 0 ? `, ${s.skipped} skipped` : '') + (s.cancelled > 0 ? `, ${s.cancelled} cancelled` : '');
    if (s.done === 0 && s.failed === 0 && s.skipped > 0 && s.cancelled === 0) {
      return s.skipped === 1
        ? 'Nothing converted: the file is already in the target format. See the list for details.'
        : `Nothing converted: ${s.skipped} files are already in the target format. See the list for details.`;
    }
    if (s.done === 0 && s.failed === 0) return `Conversion cancelled${s.skipped > 0 ? `, ${s.skipped} skipped` : ''}`;
    if (s.done === 0) {
      return s.failed === 1
        ? 'The file could not be converted. See the list for details.'
        : `No files converted. ${s.failed} failed, see the list for details.`;
    }
    if (s.failed === 0) return `${files(s.done)} converted${extra}`;
    return `${s.done} converted, ${s.failed} failed${extra}. See the list for details.`;
  },

  dropZone: {
    title: 'Drag audio files here',
    reading: 'Reading files …',
    text: 'Single files or whole folders. MP3, FLAC, WAV, AAC, Opus, Ogg and many more.',
    pickFiles: 'Choose files',
    pickFolder: 'Choose folder',
  },

  reading: {
    scanning: 'Searching folders …',
    scanningFound: (n) => `Searching folders … ${n} audio files found`,
    progress: (done, total) => `${done} of ${total} files read`,
  },

  bar: {
    format: 'Format',
    lossy: 'Lossy',
    lossless: 'Lossless',
    preset: 'Preset',
    saveIn: 'Save to',
    sourceFolder: 'Source folder',
    sourceFolderTitle: 'Same folder as the original file',
    sourceSubfolderTitle: (folder) => `Subfolder “${folder}” in the original file’s folder`,
    resetTarget: 'Reset output folder',
    resetTargetTitle: 'Back to the original file’s folder',
    active: (n) => `${n} in progress`,
    cancelAll: 'Cancel all',
    convert: 'Convert',
    convertN: (n) => `Convert ${n} files`,
    blockedNoFfmpeg: 'ffmpeg was not found. See the notice above.',
    blockedNoPreset: 'Please choose a preset.',
    blockedEmpty: 'Add files or folders first.',
    blockedAllDone: 'All files have already been converted. Add new files or remove the finished ones.',
    blockedNoAudio: 'The list contains no files with an audio track.',
    blockedAllSkipped:
      'This preset would not change any file: all are already in this format at the same or better quality. Choose a different format or preset.',
  },

  formats: {
    mp3: 'Plays on practically every device',
    aac: 'Small and good, ideal for Apple and phones',
    opus: 'Best quality at small file sizes',
    vorbis: 'Open format, e.g. for games',
    flac: 'Lossless compression, widely supported',
    alac: 'Lossless compression for Apple devices',
    wav: 'Uncompressed, maximum compatibility',
    aiff: 'Uncompressed, Apple standard',
  },

  presets: {
    none: 'No preset',
    listLabel: 'Presets',
    builtin: 'Built-in',
    own: 'My presets',
    createNew: 'New preset from current selection',
    edit: 'Edit',
    editNamed: (name) => `Edit ${name}`,
    vbr: (v) => `VBR V${v}`,
    quality: (v) => `Quality q${v}`,
    lossless: 'Lossless',
    atMost: (value) => `max. ${value}`,
    mono: 'Mono',
    stereo: 'Stereo',
    asSource: 'same as source',
    names: {
      'mp3-v0': { name: 'V0 (VBR, ~245 kbps)', description: 'Best MP3 quality with variable bitrate' },
      'mp3-v2': { name: 'V2 (VBR, ~190 kbps)', description: 'Very good quality, smaller files' },
      'mp3-320': { name: '320 kbps (CBR)', description: 'Constant bitrate, maximum compatibility' },
      'mp3-192': { name: '192 kbps (CBR)', description: 'Good compromise for car stereos and phones' },
      'mp3-128': { name: '128 kbps (CBR)', description: 'Small files, e.g. for audiobooks' },
      'aac-256': { name: '256 kbps', description: 'Very high quality, like the iTunes Store' },
      'aac-192': { name: '192 kbps' },
      'aac-128': { name: '128 kbps' },
      'opus-160': { name: '160 kbps', description: 'Practically indistinguishable from the original for music' },
      'opus-128': { name: '128 kbps' },
      'opus-96': { name: '96 kbps', description: 'Small yet good, ideal for streaming on the go' },
      'vorbis-q8': { name: 'q8 (~256 kbps)' },
      'vorbis-q6': { name: 'q6 (~192 kbps)' },
      'vorbis-q4': { name: 'q4 (~128 kbps)' },
      flac: { name: 'Same as source', description: 'Sample rate and bit depth stay unchanged' },
      'flac-16-44': { name: 'CD quality (16-bit / 44.1 kHz)', description: 'Hi-res files are reduced to CD quality' },
      alac: { name: 'Same as source', description: 'Sample rate and bit depth stay unchanged' },
      'alac-16-44': { name: 'CD quality (16-bit / 44.1 kHz)' },
      wav: { name: 'Same as source', description: 'Sample rate and bit depth stay unchanged' },
      'wav-16-44': { name: 'CD quality (16-bit / 44.1 kHz)' },
      aiff: { name: 'Same as source', description: 'Sample rate and bit depth stay unchanged' },
      'aiff-16-44': { name: 'CD quality (16-bit / 44.1 kHz)' },
    },
  },

  units: {
    kbps: (n) => `${n} kbps`,
    khz: (n) => `${n} kHz`,
    bits: (n) => `${n}-bit`,
  },

  editor: {
    titleNew: 'New preset',
    titleEdit: 'Edit preset',
    format: (label) => `Format: ${label}`,
    close: 'Close',
    name: 'Name',
    namePlaceholder: 'e.g. Car, Phone, Archive',
    mode: 'Mode',
    vbr: 'Variable (VBR)',
    cbr: 'Constant (CBR)',
    qualityLevel: 'Quality level',
    bitrate: 'Bitrate',
    quality: 'Quality',
    qualityOption: (label, kbps) => `${label} (~${kbps} kbps)`,
    compression: 'Compression',
    fastest: 'fastest',
    standard: 'default',
    smallest: 'smallest',
    sampleRate: 'Sample rate',
    bitDepth: 'Bit depth',
    channels: 'Channels',
    asSource: 'Same as source',
    atMost: (value) => `At most ${value}`,
    atMostStereo: 'At most stereo',
    mono: 'Mono',
    hint: 'Values are only ever reduced. A 44.1 kHz file stays at 44.1 kHz even if 96 kHz is selected.',
    delete: 'Delete',
    confirmDelete: 'Really delete?',
    cancel: 'Cancel',
    save: 'Save preset',
  },

  queue: {
    status: {
      ready: 'Ready',
      queued: 'Waiting',
      running: 'Running',
      done: 'Done',
      error: 'Error',
      cancelled: 'Cancelled',
      unsupported: 'No audio',
      skipped: 'Skipped',
    },
    sort: {
      added: 'Date added',
      name: 'Name',
      sampleRate: 'Sample rate',
      bitsPerSample: 'Bit depth',
      bitRate: 'Bitrate',
    },
    sortLabel: 'Sort:',
    sortAria: 'Sort by',
    ascTitle: 'Ascending, click for descending',
    descTitle: 'Descending, click for ascending',
    ascAria: 'Sorted ascending',
    descAria: 'Sorted descending',
    selectAll: 'Select all',
    selectAllTitle: (keys) => `Select all (${keys})`,
    selected: (n) => `${n} selected`,
    files,
    doneCount: (n) => `, ${n} done`,
    clearSelection: 'Clear selection',
    removeTitle: (keys) => `Remove (${keys})`,
    removeSelected: (n) => (n === 1 ? 'Remove file' : `Remove ${n} files`),
    clearFinished: 'Remove finished',
    clearAll: 'Clear list',
    listAria: 'Files',
    selectFile: (name) => `Select ${name}`,
    saved: (path) => `Saved: ${path}`,
    cancel: 'Cancel',
    remove: 'Remove',
  },

  skip: {
    sameLossless: (format) => `Already ${format} with these settings, nothing would change.`,
    notSmaller: (format) =>
      `Already ${format} at the same or a higher bitrate. Converting again would only reduce quality.`,
  },

  errors: {
    toolMissing: (tool) => `${tool || 'ffmpeg'} was not found or is not executable.`,
    io: (detail) => `File access failed: ${detail}`,
    probeParse: (detail) => `Could not parse ffprobe output: ${detail}`,
    probeFailed: (detail) => `Could not read file: ${detail}`,
    unknownFormat: () => 'Could not read file: unknown format',
    noAudio: () => 'File has no audio track',
    presetFile: (detail) => `Preset file is invalid: ${detail}`,
    userPresets: (detail) => `Could not read or save your presets: ${detail}`,
    ffmpegFailed: (detail) => detail,
    ffmpegExit: (code) => `ffmpeg exited with code ${code}`,
    cancelled: () => 'Cancelled',
    internal: (detail) => `Internal error: ${detail}`,
    fileTooLarge: () => 'The file is too large.',
    presetNameMissing: () => 'Please enter a name for the preset.',
    presetNameTooLong: () => 'The name can be at most 60 characters long.',
    presetRateMismatch: () => 'This quality setting does not match the selected format.',
    presetSampleRate: () => 'The format does not support this sample rate.',
    presetBitDepth: () => 'A bit depth can only be set for lossless formats.',
    presetChannels: () => 'Channels: only mono or stereo are possible.',
    presetBuiltinReadonly: () => 'Built-in presets cannot be changed or deleted.',
    presetDuplicateName: () => 'There is already a preset with this name for this format.',
    presetStorageMissing: () => 'No storage location found for your presets.',
    presetMissing: () => 'The selected preset no longer exists.',
    backupInvalidJson: () => 'The file is not valid JSON.',
    backupNotABackup: () => 'The file is not an Audio Konverter settings file.',
    backupNewerVersion: () => 'The file comes from a newer version of the app. Please update the app first.',
    unknown: (code) => `Unknown error (${code})`,
  },

  tray: { show: 'Show Audio Konverter', quit: 'Quit', tooltip: 'Audio Konverter' },

  fallbacks: { artist: 'Unknown Artist', album: 'Unknown Album' },

  settings: {
    title: 'Settings',
    close: 'Close settings',

    general: 'General',
    language: 'Language',
    languages: { system: 'System language', de: 'Deutsch', en: 'English' },
    mode: 'View',
    modes: { simple: 'Simple', advanced: 'Advanced' },
    modeHintSimple: 'Shows everything for everyday use, including your own presets and file names.',
    modeHintAdvanced: 'Also shows “Overwrite existing files” and ffmpeg details. These areas have a green frame.',
    hiddenActive: '“Overwrite existing files” is turned on and still applies, even though the switch is hidden here.',

    appearance: 'Appearance',
    theme: 'Color scheme',
    themes: { system: 'System', light: 'Light', dark: 'Dark' },
    density: 'List density',
    densities: { comfortable: 'Comfortable', compact: 'Compact' },
    accent: 'Accent color',
    minimizeToTray: 'Minimize to tray',
    minimizeToTrayHint:
      'When minimized, the window disappears from the taskbar. Click the tray icon to bring it back; running conversions continue.',
    accents: {
      petrol: 'Teal',
      indigo: 'Indigo',
      moss: 'Moss',
      amber: 'Amber',
      raspberry: 'Raspberry',
      graphite: 'Graphite',
    },

    output: 'Output',
    overwrite: 'Overwrite existing files',
    overwriteHint: 'When off, “(2)” is appended if a file with the same name exists.',
    subfolders: 'Create subfolders',
    subfoldersHintTarget: 'The folder structure of added folders is recreated in the output folder.',
    subfoldersHintSource: (folder) => `A subfolder “${folder}” is created next to each original file for the converted files.`,

    fileNames: 'File names',
    templateLabel: 'Template for new files. Use / to create subfolders.',
    example: (path) => `Example: ${path}`,
    resetTemplate: 'Reset',
    resetTemplateTitle: (template) => `Reset template to ${template}`,
    insertPlaceholder: (token) => `Insert ${token} at the cursor position`,
    duplicatePlaceholders: (list, n) =>
      n === 1 ? `${list} appears more than once. Is that intended?` : `${list} appear more than once. Is that intended?`,
    exampleValues: {
      name: 'Original name',
      artist: 'Artist',
      albumartist: 'Album artist',
      album: 'Album',
      track: '01',
      title: 'Title',
      disc: '1',
      year: '2024',
    },

    backup: 'Back up & transfer',
    backupHint: 'Settings and your own presets as a file, e.g. for a second computer. The output folder is not included.',
    export: 'Export …',
    import: 'Import …',
    backupFileName: 'audio-konverter-settings.json',
    backupFilter: 'Settings file',
    exported: 'Settings exported.',
    exportFailed: (error) => `Export failed: ${error}`,
    importFailed: (error) => `Import failed: ${error}`,

    about: 'About & updates',
    version: (v) => `Version ${v}`,
    autoCheck: 'Check for updates automatically',
    autoCheckHint: 'At most once a day when the app starts.',
    checkNow: 'Check for updates',
    checking: 'Checking …',
    upToDate: 'You are using the latest version.',
    updateAvailable: (v) => `Version ${v} is available.`,
    download: 'Go to download page',
    updateNow: 'Update now',
    downloadingUpdate: (v, percent) => (percent ? `Downloading version ${v} … ${percent}` : `Downloading version ${v} …`),
    installingUpdate: (v) => `Installing version ${v}. The app will restart shortly.`,
    waitForConversions: 'Available once the running conversions have finished.',
    manualUpdate:
      'This version (portable or installed as a package) does not update itself. Download the new version from the download page.',
    updateError: (error) => `Update check failed: ${error}`,
    ffmpegNote: 'Uses FFmpeg, licensed under the LGPL.',
    parallel: (n) => `Up to ${n} files at a time`,
  },

  importDialog: {
    title: 'Import settings',
    exportedAt: (date, version) => `Exported on ${date} with version ${version}`,
    settings: 'Apply settings',
    settingsHint: 'Replaces appearance, language, output and file names. The output folder stays unchanged.',
    presets: (n) => (n === 1 ? 'Import 1 preset' : `Import ${n} presets`),
    noPresets: 'The file contains no presets of your own.',
    conflicts: (n) =>
      n === 1
        ? '1 preset has the same name as an existing one. What should happen?'
        : `${n} presets have the same name as existing ones. What should happen?`,
    replace: 'Replace',
    keep: 'Keep existing',
    confirm: 'Import',
    cancel: 'Cancel',
    result: (r) => {
      const parts: string[] = [];
      if (r.settings) parts.push('settings applied');
      if (r.added) parts.push(`${presetsN(r.added)} added`);
      if (r.replaced) parts.push(`${r.replaced} replaced`);
      if (r.skipped) parts.push(`${r.skipped} skipped`);
      if (r.invalid) parts.push(`${r.invalid} invalid`);
      return parts.length ? `Import complete: ${parts.join(', ')}.` : 'Nothing was imported.';
    },
  },
};
