use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

const DEFAULT_PRESETS: &str = include_str!("../../presets/default.toml");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    Mp3,
    Aac,
    Opus,
    Vorbis,
    Flac,
    Alac,
    Wav,
    Aiff,
}

impl Codec {
    pub fn extension(self) -> &'static str {
        match self {
            Codec::Mp3 => "mp3",
            Codec::Aac | Codec::Alac => "m4a",
            Codec::Opus => "opus",
            Codec::Vorbis => "ogg",
            Codec::Flac => "flac",
            Codec::Wav => "wav",
            Codec::Aiff => "aiff",
        }
    }

    /// Anzeigename, auch als Name für Unterordner neben der Originaldatei genutzt.
    pub fn label(self) -> &'static str {
        match self {
            Codec::Mp3 => "MP3",
            Codec::Aac => "AAC",
            Codec::Opus => "Opus",
            Codec::Vorbis => "Ogg Vorbis",
            Codec::Flac => "FLAC",
            Codec::Alac => "ALAC",
            Codec::Wav => "WAV",
            Codec::Aiff => "AIFF",
        }
    }

    /// Container, in die ffmpeg ein Cover als angehängtes Bild schreiben kann.
    pub fn supports_cover(self) -> bool {
        matches!(self, Codec::Mp3 | Codec::Aac | Codec::Alac | Codec::Flac)
    }

    pub fn is_lossless(self) -> bool {
        matches!(self, Codec::Flac | Codec::Alac | Codec::Wav | Codec::Aiff)
    }

    /// Abtastraten, die der Encoder schreiben kann. Opus arbeitet immer mit 48 kHz.
    pub fn sample_rates(self) -> &'static [u32] {
        match self {
            Codec::Opus => &[],
            Codec::Mp3 => &[32000, 44100, 48000],
            Codec::Aac | Codec::Vorbis => &[44100, 48000, 88200, 96000],
            Codec::Flac | Codec::Alac | Codec::Wav | Codec::Aiff => {
                &[44100, 48000, 88200, 96000, 176400, 192000]
            }
        }
    }

    /// Zulässiger Bitratenbereich in kbit/s (verlustbehaftete Formate).
    fn bitrate_range(self) -> Option<std::ops::RangeInclusive<u32>> {
        match self {
            Codec::Mp3 => Some(32..=320),
            Codec::Aac => Some(32..=512),
            Codec::Opus => Some(6..=510),
            Codec::Vorbis => Some(45..=500),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum Rate {
    /// Qualitätsstufe des Encoders. MP3: 0–9 (kleiner ist besser), Vorbis: -1–10 (größer ist besser).
    Quality { value: f32 },
    /// Zielbitrate in kbit/s.
    Bitrate { kbps: u32 },
    /// Verlustfrei; `compression` gilt nur für FLAC (0–12).
    Lossless {
        #[serde(default)]
        compression: Option<u8>,
    },
}

/// Abtastrate, Bit-Tiefe und Kanäle sind Obergrenzen; `None` heißt „wie Quelle“.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub codec: Codec,
    pub rate: Rate,
    #[serde(default, alias = "sample_rate")]
    pub sample_rate: Option<u32>,
    #[serde(default)]
    pub channels: Option<u8>,
    #[serde(default, alias = "bit_depth")]
    pub bit_depth: Option<u8>,
    /// Mitgeliefert und nicht änderbar
    #[serde(default)]
    pub builtin: bool,
}

/// Warum ein Preset ungültig ist. `code()` dient Oberflächen zur Übersetzung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PresetError {
    #[error("Bitte einen Namen für das Preset eingeben.")]
    NameMissing,
    #[error("Der Name darf höchstens 60 Zeichen lang sein.")]
    NameTooLong,
    #[error("Diese Qualitätseinstellung passt nicht zum gewählten Format.")]
    RateMismatch,
    #[error("Diese Abtastrate unterstützt das Format nicht.")]
    SampleRateUnsupported,
    #[error("Eine Bit-Tiefe lässt sich nur bei verlustfreien Formaten festlegen.")]
    BitDepthInvalid,
    #[error("Kanäle: nur Mono oder Stereo möglich.")]
    ChannelsInvalid,
}

impl PresetError {
    pub fn code(self) -> &'static str {
        match self {
            PresetError::NameMissing => "presetNameMissing",
            PresetError::NameTooLong => "presetNameTooLong",
            PresetError::RateMismatch => "presetRateMismatch",
            PresetError::SampleRateUnsupported => "presetSampleRate",
            PresetError::BitDepthInvalid => "presetBitDepth",
            PresetError::ChannelsInvalid => "presetChannels",
        }
    }
}

impl Preset {
    /// Prüft, ob die Einstellungen zum Format passen.
    pub fn validate(&self) -> std::result::Result<(), PresetError> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(PresetError::NameMissing);
        }
        if name.chars().count() > 60 {
            return Err(PresetError::NameTooLong);
        }

        let rate_ok = match (&self.rate, self.codec) {
            (Rate::Quality { value }, Codec::Mp3) => (0.0..=9.0).contains(value),
            (Rate::Quality { value }, Codec::Vorbis) => (-1.0..=10.0).contains(value),
            (Rate::Quality { .. }, _) => false,
            (Rate::Bitrate { kbps }, codec) => codec.bitrate_range().is_some_and(|r| r.contains(kbps)),
            (Rate::Lossless { compression }, codec) => {
                codec.is_lossless() && compression.map_or(true, |level| level <= 12)
            }
        };
        if !rate_ok {
            return Err(PresetError::RateMismatch);
        }

        if let Some(rate) = self.sample_rate {
            if !self.codec.sample_rates().contains(&rate) {
                return Err(PresetError::SampleRateUnsupported);
            }
        }
        if let Some(depth) = self.bit_depth {
            if !self.codec.is_lossless() || ![16, 24, 32].contains(&depth) {
                return Err(PresetError::BitDepthInvalid);
            }
        }
        if let Some(channels) = self.channels {
            if !(1..=2).contains(&channels) {
                return Err(PresetError::ChannelsInvalid);
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct PresetFile {
    #[serde(rename = "preset", default)]
    presets: Vec<Preset>,
}

/// Liest Presets aus TOML (Format siehe `presets/default.toml`).
pub fn parse_presets(toml_text: &str) -> Result<Vec<Preset>> {
    Ok(toml::from_str::<PresetFile>(toml_text)?.presets)
}

/// Die mitgelieferten Standard-Presets.
pub fn default_presets() -> Vec<Preset> {
    let mut presets = parse_presets(DEFAULT_PRESETS).expect("eingebettete Presets müssen gültig sein");
    for p in &mut presets {
        p.builtin = true;
    }
    presets
}

/// Lädt eigene Presets (JSON). Fehlt die Datei, gibt es einfach keine.
/// Ungültige Einträge werden übersprungen statt den Start zu verhindern.
pub fn load_user_presets(path: &Path) -> Result<Vec<Preset>> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut presets: Vec<Preset> =
        serde_json::from_slice(&bytes).map_err(|e| Error::UserPresets(e.to_string()))?;
    presets.retain(|p| !p.id.is_empty() && p.validate().is_ok());
    for p in &mut presets {
        p.builtin = false;
    }
    Ok(presets)
}

/// Speichert eigene Presets atomar (erst Temp-Datei, dann umbenennen).
pub fn save_user_presets(path: &Path, presets: &[Preset]) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_vec_pretty(presets).map_err(|e| Error::UserPresets(e.to_string()))?;
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, json)?;
    std::fs::rename(&temp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_presets_are_valid_and_unique() {
        let presets = default_presets();
        assert!(presets.len() >= 8);
        let mut ids: Vec<_> = presets.iter().map(|p| p.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), presets.len(), "Preset-IDs müssen eindeutig sein");
        for p in &presets {
            assert!(p.builtin);
            assert_eq!(p.validate(), Ok(()), "Preset {} ist ungültig", p.id);
        }
    }

    #[test]
    fn user_presets_roundtrip() {
        let path = std::env::temp_dir().join(format!("audioconv-presets-{}.json", std::process::id()));
        let mut preset = default_presets().into_iter().find(|p| p.id == "flac-16-44").unwrap();
        preset.id = "eigen-1".into();
        preset.name = "Auto".into();
        preset.builtin = false;

        save_user_presets(&path, &[preset]).unwrap();
        let loaded = load_user_presets(&path).unwrap();
        std::fs::remove_file(&path).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Auto");
        assert_eq!(loaded[0].sample_rate, Some(44100));
        assert_eq!(loaded[0].bit_depth, Some(16));
    }

    #[test]
    fn rejects_mismatched_settings() {
        let mut p = default_presets().into_iter().find(|p| p.id == "mp3-320").unwrap();
        p.bit_depth = Some(24);
        assert!(p.validate().is_err());
        p.bit_depth = None;
        p.sample_rate = Some(96000);
        assert!(p.validate().is_err());
        p.sample_rate = None;
        p.rate = Rate::Bitrate { kbps: 500 };
        assert!(p.validate().is_err());
    }
}
