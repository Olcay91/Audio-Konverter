# Audio Konverter

Plattformübergreifender Audio-Konverter auf Basis von ffmpeg für Windows, macOS und Linux.
Tauri 2 (Rust) im Hintergrund, Svelte 5 mit TypeScript für die Oberfläche.

## Voraussetzungen

- **Rust** (stable) über [rustup](https://rustup.rs)
- **Node.js** 20 oder neuer
- **ffmpeg und ffprobe** im `PATH` (für die Entwicklung)
- Systemabhängigkeiten von Tauri:
  - **Windows:** Microsoft C++ Build Tools, WebView2 (bei Windows 10/11 meist vorhanden)
  - **macOS:** Xcode Command Line Tools (`xcode-select --install`)
  - **Linux (Debian/Ubuntu):**
    `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`

## Loslegen

```bash
npm install          # installiert Tauri-CLI und Frontend-Abhängigkeiten
npm run dev          # startet Vite und die App mit Hot-Reload
```

Den Kern ohne Oberfläche testen:

```bash
cargo test -p audioconv-core
cargo run -p audioconv-core --example convert -- "song.flac" mp3-v0 ./ausgabe
```

## Aufbau

```
core/      Rust-Crate ohne Tauri-Abhängigkeit
  probe.rs     ffprobe auslesen (Dauer, Codec, Tags, Cover)
  command.rs   Preset -> ffmpeg-Argumente
  runner.rs    ffmpeg ausführen, Fortschritt, Abbruch
  naming.rs    Dateinamen-Vorlagen ({artist}/{album}/{track} - {title})
  scan.rs      Ordner nach Audiodateien durchsuchen
  preset.rs    Presets (eingebettet aus presets/default.toml)
app/       Tauri-App: Befehle, Job-Queue, Events, Benachrichtigungen
ui/        Svelte-Oberfläche
presets/   Standard-Presets als TOML
scripts/   Hilfsskripte (ffmpeg für das Bundle vorbereiten)
```

Die App startet ffmpeg direkt über `tokio::process`. Abbrechen, paralleles Arbeiten und
Fortschritt (`-progress pipe:1`) laufen damit vollständig im Core und funktionieren
genauso in einer späteren CLI.

Kommunikation zwischen Oberfläche und Rust:

| Befehl / Event     | Zweck                                                   |
|--------------------|---------------------------------------------------------|
| `environment`      | ffmpeg-Pfad, Version, Anzahl paralleler Jobs            |
| `presets`          | Standard- und eigene Presets                            |
| `save_preset`, `delete_preset` | Eigene Presets anlegen, ändern, löschen     |
| `import_presets`   | Eigene Presets aus einer Einstellungsdatei übernehmen   |
| `read_text_file`, `write_text_file` | Einstellungsdatei lesen/schreiben (Import/Export) |
| `add_paths`        | Dateien/Ordner hinzufügen und mit ffprobe analysieren   |
| `remove_items`     | Einträge aus der Warteschlange entfernen                |
| `start`            | Konvertierung starten (kehrt sofort zurück)             |
| `cancel`, `cancel_all` | Jobs abbrechen                                      |
| Event `add-progress` | Fortschritt beim Einlesen (Ordner durchsuchen, ffprobe) |
| Event `job-update` | Status und Fortschritt eines Jobs                       |
| Event `batch-finished` | Zusammenfassung, wenn ein Stapel fertig ist         |

Fehler kommen als `{ code, detail? }` (`UiError` in `app/src/commands.rs`) zur Oberfläche,
übersprungene Dateien als `{ kind, codec }`. Die Texte dazu stehen in den Sprachdateien.

## Sprachen

Die Oberfläche gibt es auf Deutsch und Englisch (`ui/src/lib/i18n/`). `de.ts` ist die
Referenz, `en.ts` muss denselben Aufbau haben; TypeScript meldet fehlende Texte. Standard ist
die Systemsprache, alles außer Deutsch fällt auf Englisch zurück.

Verwendung in Komponenten: `t().bar.convert`. Weil `t()` die Spracheinstellung liest,
aktualisiert sich die Oberfläche beim Umschalten sofort.

Synchron halten:
- `presets.names` in `en.ts` ↔ IDs in `presets/default.toml` (Deutsch steht direkt in der TOML)
- `errors` in den Sprachdateien ↔ `Error::code` (core), `PresetError::code` (core) und die
  `UiError`-Codes in `app/src/commands.rs`

Systembenachrichtigungen entstehen in Rust; die Sprache wird beim Start eines Stapels mitgegeben.

## Einstellungen sichern

Einstellungen → „Sichern & übertragen“ exportiert Einstellungen und eigene Presets als JSON
(`ui/src/lib/backup.ts`, Format-Version `BACKUP_VERSION`). Der Zielordner wird nicht
exportiert, weil er an den Rechner gebunden ist. Beim Import lässt sich wählen, ob gleichnamige
Presets ersetzt oder die vorhandenen behalten werden.

## Updates

Die Update-Prüfung ist vorbereitet, aber noch aus (`ui/src/lib/updates.svelte.ts`):

1. **Einfach:** `UPDATE_URL` auf `https://api.github.com/repos/<nutzer>/<repo>/releases/latest`
   setzen. Die App vergleicht den Tag des neuesten Releases mit ihrer Version und verlinkt die
   Release-Seite.
2. **Auto-Updater:** `tauri-plugin-updater` einbinden, Signierschlüssel erzeugen
   (`npm run tauri signer generate`), signierte Builds samt `latest.json` veröffentlichen und
   `fetchLatest` durch `check()` aus `@tauri-apps/plugin-updater` ersetzen.

## Releases

Fertige Pakete baut GitHub Actions (`.github/workflows/release.yml`):

| Plattform | Pakete |
|---|---|
| Windows | NSIS-Installer (.exe), portable ZIP |
| macOS | .dmg für Apple Silicon und Intel |
| Linux | .AppImage, .deb |

Neue Version veröffentlichen:

```bash
node scripts/set-version.mjs 0.2.0      # Version in package.json, tauri.conf.json, Cargo.toml
git commit -am "Version 0.2.0"
git tag v0.2.0
git push && git push origin v0.2.0
```

Der Workflow legt einen Release-Entwurf an; nach Prüfung auf GitHub veröffentlichen.
Zum Ausprobieren ohne Release: *Actions → Release → Run workflow*, die Pakete hängen dann
als „Artifacts“ am Lauf.

Die Pakete sind nicht kostenpflichtig signiert (macOS nur ad hoc). Windows SmartScreen und
macOS Gatekeeper warnen deshalb beim ersten Start; der Release-Text erklärt, was zu tun ist.

## ffmpeg mitliefern

Für die Entwicklung reicht ffmpeg im `PATH`. In Releases liegt ein **schlanker
Audio-Build** (LGPL, nur benötigte Encoder/Muxer, alle Audio-Decoder) als Sidecar neben
der App. Gebaut wird er mit `scripts/ffmpeg/build.sh` (Linux, macOS, Windows/MSYS2), im
CI zwischengespeichert, bis sich das Skript ändert.

Die mitgelieferten Programme heißen `audiokonverter-ffmpeg` und `audiokonverter-ffprobe`,
damit z. B. das .deb-Paket nicht mit dem ffmpeg der Distribution in `/usr/bin` kollidiert.
Neben der App sucht `core/src/tools.rs` zuerst diese Namen, dann `ffmpeg`/`ffprobe`.

Lokal mit Sidecar bauen:

```bash
bash scripts/ffmpeg/build.sh                       # Ergebnis in build/ffmpeg/out/
node scripts/prepare-ffmpeg.mjs build/ffmpeg/out/ffmpeg build/ffmpeg/out/ffprobe
npm run tauri build -- --config app/tauri.ffmpeg.conf.json
```

Ohne diese Konfiguration baut `npm run build` die App ohne ffmpeg; sie nutzt dann das
installierte ffmpeg des Systems.

### Lizenzhinweise

- ffmpeg als **LGPL-Build** und als separates Programm mitliefern; Hinweise und
  Quellverweise stehen in `THIRD_PARTY_NOTICES.md` (wird in die Pakete übernommen) und
  in `FFMPEG-BUILD.txt` (portable ZIP).
- `libfdk_aac` ist nicht frei weiterverteilbar und wird nicht verwendet; AAC nutzt den
  eingebauten ffmpeg-Encoder.
- Die eigene Lizenz des Projekts (z. B. MIT oder GPL-3.0) ist noch festzulegen.

## Presets

In der unteren Leiste wählt man zuerst das **Format** (MP3, FLAC, …) und daneben ein
**Preset** mit den gängigen Einstellungen dieses Formats. Über „Neues Preset aus
aktueller Auswahl“ lassen sich eigene Presets mit eigenem Namen anlegen, bearbeiten
und löschen.

- Standard-Presets: `presets/default.toml`, werden beim Kompilieren eingebettet.
- Eigene Presets: `presets.json` im Konfigurationsordner der App
  (Windows: `%APPDATA%\org.audiokonverter.desktop`, macOS:
  `~/Library/Application Support/org.audiokonverter.desktop`, Linux:
  `~/.config/org.audiokonverter.desktop`).

Abtastrate, Bit-Tiefe und Kanäle sind Obergrenzen: Quellen darunter bleiben
unverändert, hochgerechnet wird nie.
