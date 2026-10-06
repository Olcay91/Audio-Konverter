#!/usr/bin/env bash
# Baut ein schlankes, statisches ffmpeg + ffprobe nur für Audio, unter der LGPL.
#
#   bash scripts/ffmpeg/build.sh
#
# Läuft unter Linux, macOS und Windows (MSYS2, Umgebung UCRT64). Ergebnis in
# build/ffmpeg/out/: ffmpeg, ffprobe (.exe unter Windows), FFMPEG-BUILD.txt.
#
# USE_SYSTEM_LIBS=1: LAME, Ogg, Vorbis und Opus nicht selbst bauen, sondern statische
# Bibliotheken des Systems nutzen (unter MSYS2: Pakete mingw-w64-ucrt-x86_64-lame,
# -libogg, -libvorbis, -opus). Sonst werden sie aus den Quellen gebaut.
#
# Enthalten ist nur, was die App braucht (siehe core/src/command.rs):
#   - Lesen: alle Container und alle Audio-Decoder, damit „viele Formate“ stimmt
#   - Schreiben: libmp3lame, aac, libopus, libvorbis, flac, alac, PCM (WAV/AIFF)
#   - Filter für Abtastrate, Kanäle und Sampleformat (+ loudnorm für später)
# Keine GPL-Teile (kein --enable-gpl), kein libfdk_aac, keine Video-Encoder.
#
# Versionen hier anpassen; der CI-Cache hängt am Inhalt dieser Datei.

set -euo pipefail

FFMPEG_VERSION=7.1.1
LAME_VERSION=3.100
OGG_VERSION=1.3.5
VORBIS_VERSION=1.3.7
OPUS_VERSION=1.5.2

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
WORK=${WORK:-$ROOT/build/ffmpeg}
SRC=$WORK/src
PREFIX=$WORK/prefix
OUT=${OUT:-$WORK/out}
JOBS=$(getconf _NPROCESSORS_ONLN 2>/dev/null || nproc 2>/dev/null || echo 4)

case "$(uname -s)" in
  MINGW* | MSYS*) OS=windows ;;
  Darwin) OS=macos ;;
  *) OS=linux ;;
esac

EXE=""
THREADS=pthreads
EXTRA_LDFLAGS=""
if [ "$OS" = windows ]; then
  EXE=".exe"
  THREADS=w32threads
  # Vollständig statisch, damit keine MinGW-DLLs mitgeliefert werden müssen
  EXTRA_LDFLAGS="-static"
fi
if [ "$OS" = macos ]; then
  export MACOSX_DEPLOYMENT_TARGET=${MACOSX_DEPLOYMENT_TARGET:-11.0}
fi

mkdir -p "$SRC" "$PREFIX" "$OUT"
export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"

log() { printf '\n==> %s\n' "$*"; }

# Lädt ein Archiv (einmal) und entpackt es frisch.
fetch() {
  local url=$1 dir=$2
  local file="$SRC/$(basename "$url")"
  if [ ! -f "$file" ]; then
    log "Lade $(basename "$url")"
    curl -fL --retry 3 --retry-delay 5 -o "$file.tmp" "$url"
    mv "$file.tmp" "$file"
  fi
  rm -rf "${SRC:?}/$dir"
  tar -xf "$file" -C "$SRC"
}

# Statische Bibliothek mit autotools bauen und nach $PREFIX installieren.
autotools() {
  local dir=$1
  shift
  log "Baue $dir"
  (
    cd "$SRC/$dir"
    CFLAGS="-O2 -I$PREFIX/include" LDFLAGS="-L$PREFIX/lib" \
      ./configure --prefix="$PREFIX" --disable-shared --enable-static "$@"
    make -j"$JOBS"
    make install
  )
}

# ---------- Bibliotheken ----------

if [ "${USE_SYSTEM_LIBS:-0}" != 1 ]; then

fetch "https://downloads.sourceforge.net/project/lame/lame/$LAME_VERSION/lame-$LAME_VERSION.tar.gz" "lame-$LAME_VERSION"
autotools "lame-$LAME_VERSION" --disable-frontend --disable-decoder

fetch "https://downloads.xiph.org/releases/ogg/libogg-$OGG_VERSION.tar.xz" "libogg-$OGG_VERSION"
autotools "libogg-$OGG_VERSION"

fetch "https://downloads.xiph.org/releases/vorbis/libvorbis-$VORBIS_VERSION.tar.xz" "libvorbis-$VORBIS_VERSION"
# Veraltete Compiler-Option, an der aktuelle Clang-Versionen unter macOS scheitern
sed -i.bak 's/-force_cpusubtype_ALL//g' "$SRC/libvorbis-$VORBIS_VERSION/configure"
autotools "libvorbis-$VORBIS_VERSION" --disable-examples --disable-docs --disable-oggtest \
  --with-ogg="$PREFIX"

fetch "https://downloads.xiph.org/releases/opus/opus-$OPUS_VERSION.tar.gz" "opus-$OPUS_VERSION"
autotools "opus-$OPUS_VERSION" --disable-doc --disable-extra-programs

else
  log "Nutze LAME, Ogg, Vorbis und Opus des Systems"
  LAME_VERSION="$LAME_VERSION (System)"
  OGG_VERSION="$(pkg-config --modversion ogg) (System)"
  VORBIS_VERSION="$(pkg-config --modversion vorbis) (System)"
  OPUS_VERSION="$(pkg-config --modversion opus) (System)"
fi

# ---------- ffmpeg ----------

fetch "https://ffmpeg.org/releases/ffmpeg-$FFMPEG_VERSION.tar.xz" "ffmpeg-$FFMPEG_VERSION"
cd "$SRC/ffmpeg-$FFMPEG_VERSION"

# Alle Audio-Decoder dieser ffmpeg-Version, dazu mjpeg/bmp für eingebettete Cover.
AUDIO_DECODERS='^(aac.*|ac3.*|eac3|alac|als|amrnb|amrwb|ape|apac|atrac.*|binkaudio_.*|cook|dca|dolby_e|dsd_.*|dst|flac|g723_1|g729|gsm.*|iac|imc|metasound|mlp|truehd|mp1.*|mp2.*|mp3.*|mpc7|mpc8|nellymoser|on2avc|opus|osq|qdm2|qdmc|qoa|ra_144|ra_288|ralf|s302m|shorten|sipr|speex|tak|truespeech|tta|twinvq|vorbis|wavpack|wma.*|xma1|xma2|pcm_.*|adpcm_.*|mjpeg|bmp)$'
# Ohne Hardware-/Plattform-Decoder (AudioToolbox, MediaCodec, Media Foundation …)
DECODERS=$(./configure --list-decoders | tr -s ' \t' '\n' | grep -E "$AUDIO_DECODERS" |
  grep -vE '_(at|mediacodec|mf|qsv|cuvid)$' | paste -sd, -)

ENCODERS=libmp3lame,aac,libopus,libvorbis,flac,alac,pcm_s16le,pcm_s24le,pcm_s32le,pcm_s16be,pcm_s24be,pcm_s32be
MUXERS=mp3,ipod,mp4,opus,ogg,flac,wav,aiff,null
FILTERS=abuffer,abuffersink,aformat,anull,aresample,buffer,buffersink,format,null,volume,loudnorm,ebur128

CONFIGURE_FLAGS=(
  --prefix="$PREFIX"
  --pkg-config-flags=--static
  --extra-cflags="-I$PREFIX/include"
  --extra-ldflags="-L$PREFIX/lib $EXTRA_LDFLAGS"
  --disable-everything
  --disable-autodetect
  --disable-shared --enable-static
  --disable-doc --disable-debug --disable-network --disable-ffplay
  --enable-ffmpeg --enable-ffprobe
  --enable-"$THREADS"
  --enable-libmp3lame --enable-libopus --enable-libvorbis
  --enable-protocol=file,pipe
  --enable-demuxers --enable-parsers --enable-bsfs
  --enable-decoder="$DECODERS"
  --enable-encoder="$ENCODERS"
  --enable-muxer="$MUXERS"
  --enable-filter="$FILTERS"
)

log "Konfiguriere ffmpeg $FFMPEG_VERSION"
./configure "${CONFIGURE_FLAGS[@]}"
log "Baue ffmpeg"
make -j"$JOBS"
make install

cp "$PREFIX/bin/ffmpeg$EXE" "$PREFIX/bin/ffprobe$EXE" "$OUT/"

# ---------- Prüfen ----------

FFMPEG="$OUT/ffmpeg$EXE"
FFPROBE="$OUT/ffprobe$EXE"

log "Prüfe Lizenz"
if "$FFMPEG" -hide_banner -buildconf | grep -q -- '--enable-gpl'; then
  echo "Fehler: Build enthält GPL-Teile." >&2
  exit 1
fi

log "Probelauf aller Zielformate"
TEST=$WORK/test
rm -rf "$TEST" && mkdir -p "$TEST"
# 1 Sekunde Stille, 44,1 kHz Stereo, als Rohdaten über stdin
head -c 176400 /dev/zero >"$TEST/silence.raw"
for target in "mp3:libmp3lame" "m4a:aac" "opus:libopus" "ogg:libvorbis" "flac:flac" "alac.m4a:alac" "wav:pcm_s16le" "aiff:pcm_s16be"; do
  ext=${target%%:*}
  codec=${target#*:}
  "$FFMPEG" -hide_banner -loglevel error -y -f s16le -ar 44100 -ac 2 -i "$TEST/silence.raw" \
    -c:a "$codec" "$TEST/test.$ext"
  "$FFPROBE" -v error -show_entries stream=codec_name -of csv=p=0 "$TEST/test.$ext" | grep -q . || {
    echo "Fehler: $ext ($codec) lässt sich nicht lesen" >&2
    exit 1
  }
  echo "  ok  $ext ($codec)"
done

# ---------- Nachweis für die Lizenzhinweise ----------

{
  echo "ffmpeg $FFMPEG_VERSION (LGPL v2.1 oder neuer), gebaut mit scripts/ffmpeg/build.sh"
  echo "Quelle: https://ffmpeg.org/releases/ffmpeg-$FFMPEG_VERSION.tar.xz"
  echo
  echo "Enthaltene Bibliotheken:"
  echo "  LAME $LAME_VERSION (LGPL)          https://lame.sourceforge.io"
  echo "  libogg $OGG_VERSION (BSD)          https://xiph.org/ogg/"
  echo "  libvorbis $VORBIS_VERSION (BSD)    https://xiph.org/vorbis/"
  echo "  Opus $OPUS_VERSION (BSD)           https://opus-codec.org"
  echo
  echo "Build-Konfiguration:"
  "$FFMPEG" -hide_banner -buildconf
} >"$OUT/FFMPEG-BUILD.txt"

log "Fertig: $OUT"
ls -l "$OUT"
