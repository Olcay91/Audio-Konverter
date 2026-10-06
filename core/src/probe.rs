use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::tools::{command, Tools};
use crate::{Error, Result};

/// Das Wichtigste über eine Quelldatei, für Anzeige und Konvertierung.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub format_name: String,
    pub duration_secs: Option<f64>,
    pub codec: String,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub bit_rate: Option<u64>,
    pub bits_per_sample: Option<u32>,
    pub has_cover: bool,
    /// Tags mit kleingeschriebenen Schlüsseln (artist, album, title, track, …).
    pub tags: BTreeMap<String, String>,
}

/// Liest eine Datei mit ffprobe aus. Liefert [`Error::NoAudio`], wenn keine Audiospur existiert.
pub async fn probe(tools: &Tools, input: &Path) -> Result<MediaInfo> {
    let out = command(&tools.ffprobe)
        .args(["-v", "error", "-print_format", "json", "-show_format", "-show_streams"])
        .arg(input)
        .output()
        .await
        .map_err(|source| Error::ToolMissing { tool: "ffprobe", source })?;

    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if msg.is_empty() { Error::UnknownFormat } else { Error::ProbeFailed(msg) });
    }
    parse_probe(&out.stdout)
}

#[derive(Deserialize)]
struct ProbeOutput {
    #[serde(default)]
    streams: Vec<ProbeStream>,
    format: Option<ProbeFormat>,
}

#[derive(Deserialize)]
struct ProbeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
    bit_rate: Option<String>,
    bits_per_sample: Option<u32>,
    bits_per_raw_sample: Option<String>,
    duration: Option<String>,
    #[serde(default)]
    disposition: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    tags: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct ProbeFormat {
    format_name: Option<String>,
    duration: Option<String>,
    bit_rate: Option<String>,
    #[serde(default)]
    tags: BTreeMap<String, String>,
}

pub(crate) fn parse_probe(json: &[u8]) -> Result<MediaInfo> {
    let parsed: ProbeOutput = serde_json::from_slice(json)?;
    let audio = parsed
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("audio"))
        .ok_or(Error::NoAudio)?;
    let format = parsed.format.as_ref();

    let has_cover = parsed.streams.iter().any(|s| {
        s.codec_type.as_deref() == Some("video")
            && s.disposition.get("attached_pic").and_then(|v| v.as_i64()) == Some(1)
    });

    let duration_secs = format
        .and_then(|f| f.duration.as_deref())
        .or(audio.duration.as_deref())
        .and_then(|d| d.parse::<f64>().ok())
        .filter(|d| *d > 0.0);

    // Stream-Tags zuerst (Ogg/Opus speichern dort), Container-Tags überschreiben.
    let mut tags = BTreeMap::new();
    for (key, value) in audio
        .tags
        .iter()
        .chain(format.map(|f| f.tags.iter()).into_iter().flatten())
    {
        tags.insert(key.to_lowercase(), value.clone());
    }

    let bits_per_sample = audio
        .bits_per_raw_sample
        .as_deref()
        .and_then(|b| b.parse::<u32>().ok())
        .filter(|b| *b > 0)
        .or(audio.bits_per_sample.filter(|b| *b > 0));

    Ok(MediaInfo {
        format_name: format.and_then(|f| f.format_name.clone()).unwrap_or_default(),
        duration_secs,
        codec: audio.codec_name.clone().unwrap_or_default(),
        sample_rate: audio.sample_rate.as_deref().and_then(|s| s.parse().ok()),
        channels: audio.channels,
        bit_rate: audio
            .bit_rate
            .as_deref()
            .or(format.and_then(|f| f.bit_rate.as_deref()))
            .and_then(|b| b.parse().ok()),
        bits_per_sample,
        has_cover,
        tags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLAC_WITH_COVER: &str = r#"{
      "streams": [
        {"codec_type":"audio","codec_name":"flac","sample_rate":"96000","channels":2,
         "bits_per_raw_sample":"24","tags":{}},
        {"codec_type":"video","codec_name":"mjpeg","disposition":{"default":0,"attached_pic":1}}
      ],
      "format": {"format_name":"flac","duration":"215.400000","bit_rate":"3100000",
                 "tags":{"ARTIST":"Beispiel","ALBUM":"Album","TITLE":"Titel","track":"3/12"}}
    }"#;

    #[test]
    fn parses_flac_with_cover() {
        let info = parse_probe(FLAC_WITH_COVER.as_bytes()).unwrap();
        assert_eq!(info.codec, "flac");
        assert_eq!(info.sample_rate, Some(96000));
        assert_eq!(info.bits_per_sample, Some(24));
        assert!(info.has_cover);
        assert_eq!(info.tags.get("artist").map(String::as_str), Some("Beispiel"));
        assert!((info.duration_secs.unwrap() - 215.4).abs() < 1e-6);
    }

    #[test]
    fn rejects_files_without_audio() {
        let json = r#"{"streams":[{"codec_type":"video","codec_name":"png"}],"format":{}}"#;
        assert!(matches!(parse_probe(json.as_bytes()), Err(Error::NoAudio)));
    }
}
