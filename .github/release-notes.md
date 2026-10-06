## Downloads

| System | Datei |
|---|---|
| Windows (Installer) | `Audio Konverter_…_x64-setup.exe` |
| Windows (portabel, ohne Installation) | `Audio-Konverter-…-windows-x64-portable.zip` |
| macOS mit Apple-Chip (M1 und neuer) | `Audio Konverter_…_aarch64.dmg` |
| macOS mit Intel-Prozessor | `Audio Konverter_…_x64.dmg` |
| Linux (portabel) | `audio-konverter_…_amd64.AppImage` |
| Linux (Debian, Ubuntu, Mint) | `audio-konverter_…_amd64.deb` |

ffmpeg ist bereits enthalten, es muss nichts zusätzlich installiert werden.

## Hinweise zum ersten Start

Die App ist (noch) nicht mit einem kostenpflichtigen Zertifikat signiert. Deshalb warnt das System beim ersten Start:

- **Windows:** „Der Computer wurde durch Windows geschützt“ → *Weitere Informationen* → *Trotzdem ausführen*. Die portable Version braucht die WebView2-Laufzeit, die unter Windows 10/11 meist schon installiert ist.
- **macOS:** Rechtsklick auf die App → *Öffnen* → *Öffnen*. Ab macOS 15 stattdessen: *Systemeinstellungen → Datenschutz & Sicherheit → Trotzdem öffnen*.
- **Linux (AppImage):** Datei ausführbar machen (`chmod +x`) und starten.

---

*English:* ffmpeg is included. The app is not code-signed yet, so Windows SmartScreen and macOS Gatekeeper will warn on first launch (Windows: *More info → Run anyway*; macOS: right-click → *Open*).
