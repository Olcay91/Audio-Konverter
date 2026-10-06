# Drittsoftware / Third-party software

Audio Konverter liefert in seinen fertigen Paketen ffmpeg und ffprobe als eigenständige
Programme mit. Sie werden von der App nur aufgerufen, nicht eingebunden.

## FFmpeg

- Lizenz: GNU Lesser General Public License (LGPL) v2.1 oder neuer
- Projekt: https://ffmpeg.org
- Quellcode: https://ffmpeg.org/releases/ (Version siehe `FFMPEG-BUILD.txt` bzw. `ffmpeg -version`)
- Build: schlanker Audio-Build ohne GPL-Teile, erstellt mit
  [`scripts/ffmpeg/build.sh`](scripts/ffmpeg/build.sh) aus diesem Repository.
  Damit lässt sich ffmpeg aus den unveränderten Originalquellen nachbauen oder durch
  eine eigene Version ersetzen (Datei `audiokonverter-ffmpeg` neben der App austauschen).

FFmpeg ist eine Marke von Fabrice Bellard, dem Gründer des FFmpeg-Projekts.

## In ffmpeg enthaltene Bibliotheken

| Bibliothek | Lizenz | Projekt |
|---|---|---|
| LAME (MP3) | LGPL v2 oder neuer | https://lame.sourceforge.io |
| Opus | BSD (3-Klausel) | https://opus-codec.org |
| libvorbis | BSD (3-Klausel) | https://xiph.org/vorbis/ |
| libogg | BSD (3-Klausel) | https://xiph.org/ogg/ |

Die vollständigen Lizenztexte stehen in den Quellarchiven der jeweiligen Projekte.
