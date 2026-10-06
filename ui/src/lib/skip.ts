// Vorhersage, ob eine Datei übersprungen würde. Spiegel von `skip_reason` in
// core/src/command.rs – bei Änderungen dort hier mitziehen.
// Rust entscheidet beim Start weiterhin selbst; das hier dient nur der Oberfläche
// (z. B. Konvertieren-Button sperren, wenn es nichts zu tun gibt).
import { MP3_VBR_KBPS, VORBIS_KBPS, type Codec, type MediaInfo, type Preset, type SkipReason } from './api';

const LOSSLESS: Codec[] = ['flac', 'alac', 'wav', 'aiff'];

/** Format der Quelle wie `source_codec` in Rust */
function sourceCodec(info: MediaInfo): Codec | null {
  const c = info.codec;
  if (c === 'mp3' || c === 'aac' || c === 'opus' || c === 'vorbis' || c === 'flac' || c === 'alac') return c;
  if (c.startsWith('pcm_') && info.formatName.includes('wav')) return 'wav';
  if (c.startsWith('pcm_') && info.formatName.includes('aiff')) return 'aiff';
  return null;
}

/** Würde das Preset Abtastrate, Bit-Tiefe oder Kanäle verringern? */
function reduces(preset: Preset, info: MediaInfo): boolean {
  const lower = (target: number | null, source: number | null) => target != null && source != null && source > target;
  return (
    lower(preset.sampleRate, info.sampleRate) ||
    lower(preset.bitDepth, info.bitsPerSample) ||
    lower(preset.channels, info.channels)
  );
}

/** Ungefähre Bitrate einer VBR-Qualitätsstufe wie `estimated_kbps` in Rust */
function estimatedKbps(codec: Codec, quality: number): number | null {
  const q = Math.round(quality);
  if (codec === 'mp3') return MP3_VBR_KBPS[Math.min(9, Math.max(0, q))];
  if (codec === 'vorbis') {
    const v = Math.min(10, Math.max(-1, q));
    return v === -1 ? 45 : VORBIS_KBPS[v];
  }
  return null;
}

export function skipReason(preset: Preset, info: MediaInfo): SkipReason | null {
  const source = sourceCodec(info);
  if (!source || source !== preset.codec || reduces(preset, info)) return null;
  if (LOSSLESS.includes(source)) return { kind: 'sameLossless', codec: source };

  if (!info.bitRate) return null;
  const sourceKbps = info.bitRate / 1000;
  const rate = preset.rate;
  const targetKbps =
    rate.mode === 'bitrate' ? rate.kbps : rate.mode === 'quality' ? estimatedKbps(source, rate.value) : null;
  if (targetKbps == null) return null;
  // Erst ab spürbar niedrigerer Bitrate lohnt sich das Verkleinern.
  return targetKbps >= sourceKbps * 0.9 ? { kind: 'notSmaller', codec: source } : null;
}
