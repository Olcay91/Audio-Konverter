use std::ffi::OsString;
use std::path::Path;

use serde::Serialize;

use crate::preset::{Codec, Preset, Rate};
use crate::probe::MediaInfo;

/// Baut die ffmpeg-Argumente für einen Job. Metadaten werden immer übernommen,
/// das Cover nur, wenn Quelle und Zielformat eines haben bzw. unterstützen.
/// Fortschritt landet als `key=value`-Zeilen auf stdout (`-progress pipe:1`).
///
/// Abtastrate, Bit-Tiefe und Kanäle aus dem Preset sind Obergrenzen: Liegt die
/// Quelle darunter, bleibt sie unverändert, damit nichts sinnlos hochgerechnet wird.
pub fn build_args(input: &Path, output: &Path, preset: &Preset, info: &MediaInfo) -> Vec<OsString> {
    let mut a: Vec<OsString> = Vec::new();
    macro_rules! push {
        ($($x:expr),+ $(,)?) => {{ $( a.push(OsString::from($x)); )+ }};
    }

    push!("-hide_banner", "-nostdin", "-nostats", "-loglevel", "error", "-y");
    push!("-i", input);
    push!("-map", "0:a:0");

    let codec = preset.codec;
    let bit_depth = preset
        .bit_depth
        .filter(|target| info.bits_per_sample.map_or(true, |src| src > u32::from(*target)));
    let sample_rate = preset
        .sample_rate
        .filter(|target| info.sample_rate.map_or(true, |src| src > *target));

    if info.has_cover && codec.supports_cover() {
        push!("-map", "0:v:0?", "-c:v", "copy", "-disposition:v:0", "attached_pic");
    }
    push!("-map_metadata", "0");

    match codec {
        Codec::Mp3 => push!("-c:a", "libmp3lame"),
        Codec::Aac => push!("-c:a", "aac"),
        Codec::Opus => push!("-c:a", "libopus"),
        Codec::Vorbis => push!("-c:a", "libvorbis"),
        Codec::Flac => push!("-c:a", "flac"),
        Codec::Alac => push!("-c:a", "alac"),
        Codec::Wav => push!("-c:a", pcm_codec(pcm_depth(bit_depth, info), false)),
        Codec::Aiff => push!("-c:a", pcm_codec(pcm_depth(bit_depth, info), true)),
    }

    match &preset.rate {
        Rate::Quality { value } => push!("-q:a", format_number(*value)),
        Rate::Bitrate { kbps } => push!("-b:a", format!("{kbps}k")),
        Rate::Lossless { compression } => {
            if let (Codec::Flac, Some(level)) = (codec, compression) {
                push!("-compression_level", level.to_string());
            }
        }
    }

    if let Some(depth) = bit_depth {
        match codec {
            Codec::Flac if depth <= 16 => push!("-sample_fmt", "s16"),
            Codec::Flac => push!("-sample_fmt", "s32", "-bits_per_raw_sample", depth.to_string()),
            Codec::Alac if depth <= 16 => push!("-sample_fmt", "s16p"),
            Codec::Alac => push!("-sample_fmt", "s32p"),
            _ => {} // PCM: über die Codec-Wahl geregelt, verlustbehaftet: ohne Bedeutung
        }
    }

    if let Some(rate) = sample_rate {
        push!("-ar", rate.to_string());
    }
    if let Some(channels) = preset.channels.filter(|c| info.channels.map_or(true, |src| src > u32::from(*c))) {
        push!("-ac", channels.to_string());
    }

    match codec {
        Codec::Mp3 => push!("-id3v2_version", "3", "-write_id3v1", "1"),
        Codec::Aac | Codec::Alac => push!("-movflags", "+faststart"),
        Codec::Wav => push!("-rf64", "auto"),
        _ => {}
    }

    push!("-progress", "pipe:1");
    push!(output);
    a
}

/// Warum eine Datei übersprungen wird. Wird als `{ "kind": "...", "codec": "..." }`
/// an Oberflächen geschickt, die den Text selbst übersetzen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SkipReason {
    /// Gleiches verlustfreies Format ohne Reduzierung: Ergebnis wäre bitgleich.
    SameLossless { codec: Codec },
    /// Gleiches verlustbehaftetes Format ohne spürbar kleinere Bitrate.
    NotSmaller { codec: Codec },
}

impl std::fmt::Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkipReason::SameLossless { codec } => {
                write!(f, "Ist bereits {} mit diesen Einstellungen, es würde sich nichts ändern.", codec.label())
            }
            SkipReason::NotSmaller { codec } => write!(
                f,
                "Ist bereits {} mit gleicher oder höherer Bitrate. Erneutes Umwandeln würde die Qualität nur verschlechtern.",
                codec.label()
            ),
        }
    }
}

/// Liefert einen Grund, wenn die Konvertierung nichts bringen würde:
/// - verlustfrei → gleiches verlustfreies Format ohne Reduzierung (bitgleiches Ergebnis)
/// - verlustbehaftet → gleiches Format mit gleicher oder höherer Bitrate
///   (erneutes Kodieren würde die Qualität nur verschlechtern)
pub fn skip_reason(preset: &Preset, info: &MediaInfo) -> Option<SkipReason> {
    let source = source_codec(info)?;
    if source != preset.codec || reduces(preset, info) {
        return None;
    }
    if source.is_lossless() {
        return Some(SkipReason::SameLossless { codec: source });
    }

    let source_kbps = info.bit_rate? as f64 / 1000.0;
    let target_kbps = match &preset.rate {
        Rate::Bitrate { kbps } => *kbps as f64,
        Rate::Quality { value } => estimated_kbps(source, *value)?,
        Rate::Lossless { .. } => return None,
    };
    // Erst ab spürbar niedrigerer Bitrate lohnt sich das Verkleinern.
    (target_kbps >= source_kbps * 0.9).then_some(SkipReason::NotSmaller { codec: source })
}

/// Reduziert das Preset Abtastrate, Bit-Tiefe oder Kanäle dieser Quelle?
fn reduces(preset: &Preset, info: &MediaInfo) -> bool {
    let lower = |target: Option<u32>, source: Option<u32>| matches!((target, source), (Some(t), Some(s)) if s > t);
    lower(preset.sample_rate, info.sample_rate)
        || lower(preset.bit_depth.map(u32::from), info.bits_per_sample)
        || lower(preset.channels.map(u32::from), info.channels)
}

fn source_codec(info: &MediaInfo) -> Option<Codec> {
    let format = info.format_name.as_str();
    Some(match info.codec.as_str() {
        "mp3" => Codec::Mp3,
        "aac" => Codec::Aac,
        "opus" => Codec::Opus,
        "vorbis" => Codec::Vorbis,
        "flac" => Codec::Flac,
        "alac" => Codec::Alac,
        c if c.starts_with("pcm_") && format.contains("wav") => Codec::Wav,
        c if c.starts_with("pcm_") && format.contains("aiff") => Codec::Aiff,
        _ => return None,
    })
}

/// Ungefähre Bitrate einer VBR-Qualitätsstufe.
fn estimated_kbps(codec: Codec, quality: f32) -> Option<f64> {
    const MP3: [f64; 10] = [245.0, 225.0, 190.0, 175.0, 165.0, 130.0, 115.0, 100.0, 85.0, 65.0];
    const VORBIS: [f64; 12] = [45.0, 64.0, 80.0, 96.0, 112.0, 128.0, 160.0, 192.0, 224.0, 256.0, 320.0, 500.0];
    let q = quality.round() as i32;
    match codec {
        Codec::Mp3 => MP3.get(q.clamp(0, 9) as usize).copied(),
        Codec::Vorbis => VORBIS.get((q.clamp(-1, 10) + 1) as usize).copied(),
        _ => None,
    }
}

/// Bit-Tiefe für WAV/AIFF: Vorgabe, sonst wie Quelle (mind. 16 Bit).
fn pcm_depth(bit_depth: Option<u8>, info: &MediaInfo) -> u8 {
    bit_depth.unwrap_or(match info.bits_per_sample {
        Some(b) if b > 24 => 32,
        Some(b) if b > 16 => 24,
        _ => 16,
    })
}

fn pcm_codec(depth: u8, big_endian: bool) -> String {
    let bits = match depth {
        0..=16 => 16,
        17..=24 => 24,
        _ => 32,
    };
    format!("pcm_s{bits}{}", if big_endian { "be" } else { "le" })
}

fn format_number(v: f32) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        v.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::default_presets;
    use std::collections::BTreeMap;

    fn info(bits: Option<u32>, cover: bool) -> MediaInfo {
        MediaInfo {
            format_name: "flac".into(),
            duration_secs: Some(10.0),
            codec: "flac".into(),
            sample_rate: Some(96000),
            channels: Some(2),
            bit_rate: None,
            bits_per_sample: bits,
            has_cover: cover,
            tags: BTreeMap::new(),
        }
    }

    fn args_for(id: &str, info: &MediaInfo) -> Vec<String> {
        args_with(id, info, None, None)
    }

    fn args_with(id: &str, info: &MediaInfo, rate: Option<u32>, depth: Option<u8>) -> Vec<String> {
        let mut preset = default_presets().into_iter().find(|p| p.id == id).unwrap();
        preset.sample_rate = rate;
        preset.bit_depth = depth;
        build_args(Path::new("in.flac"), Path::new("out.x"), &preset, info)
            .into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect()
    }

    fn has_pair(args: &[String], key: &str, value: &str) -> bool {
        args.windows(2).any(|w| w[0] == key && w[1] == value)
    }

    #[test]
    fn mp3_v0_keeps_cover_and_uses_vbr() {
        let a = args_for("mp3-v0", &info(Some(24), true));
        assert!(has_pair(&a, "-c:a", "libmp3lame"));
        assert!(has_pair(&a, "-q:a", "0"));
        assert!(has_pair(&a, "-c:v", "copy"));
        assert_eq!(a.last().unwrap(), "out.x");
    }

    #[test]
    fn opus_drops_cover() {
        let a = args_for("opus-160", &info(Some(16), true));
        assert!(!a.iter().any(|s| s == "-c:v"));
        assert!(has_pair(&a, "-b:a", "160k"));
    }

    #[test]
    fn wav_follows_source_depth() {
        assert!(has_pair(&args_for("wav", &info(Some(24), false)), "-c:a", "pcm_s24le"));
        assert!(has_pair(&args_with("wav", &info(Some(24), false), None, Some(16)), "-c:a", "pcm_s16le"));
        assert!(has_pair(&args_for("aiff", &info(None, false)), "-c:a", "pcm_s16be"));
    }

    #[test]
    fn flac_cd_quality_resamples() {
        let a = args_with("flac", &info(Some(24), false), Some(44100), Some(16));
        assert!(has_pair(&a, "-sample_fmt", "s16"));
        assert!(has_pair(&a, "-ar", "44100"));
    }

    fn preset(id: &str) -> Preset {
        default_presets().into_iter().find(|p| p.id == id).unwrap()
    }

    #[test]
    fn skips_identical_lossless() {
        let cd = MediaInfo { sample_rate: Some(44100), ..info(Some(16), false) };
        assert!(skip_reason(&preset("flac"), &cd).is_some());
        assert!(skip_reason(&preset("flac-16-44"), &cd).is_some());
        // Hi-Res wird reduziert -> konvertieren
        assert!(skip_reason(&preset("flac-16-44"), &info(Some(24), false)).is_none());
        // anderes Format -> konvertieren
        assert!(skip_reason(&preset("alac"), &cd).is_none());
    }

    #[test]
    fn skips_lossy_without_smaller_bitrate() {
        let mp3 = |kbps: u64| MediaInfo {
            format_name: "mp3".into(),
            codec: "mp3".into(),
            sample_rate: Some(44100),
            bit_rate: Some(kbps * 1000),
            ..info(None, false)
        };
        assert!(skip_reason(&preset("mp3-320"), &mp3(128)).is_some());
        assert!(skip_reason(&preset("mp3-320"), &mp3(320)).is_some());
        assert!(skip_reason(&preset("mp3-128"), &mp3(320)).is_none());
        assert!(skip_reason(&preset("mp3-v0"), &mp3(320)).is_none());
    }

    #[test]
    fn never_upsamples() {
        // Quelle: 96 kHz / 24 Bit, Grenze 192 kHz / 24 Bit -> nichts ändern
        let a = args_with("flac", &info(Some(24), false), Some(192000), Some(24));
        assert!(!a.iter().any(|s| s == "-ar" || s == "-sample_fmt"));
        // Quelle 16 Bit, Grenze 24 Bit -> WAV bleibt 16 Bit
        assert!(has_pair(&args_with("wav", &info(Some(16), false), None, Some(24)), "-c:a", "pcm_s16le"));
    }
}
