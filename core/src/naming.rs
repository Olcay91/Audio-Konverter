//! Zieldateinamen aus Vorlagen wie `{artist}/{album}/{track} - {title}`.

use std::path::{Path, PathBuf};

use crate::probe::MediaInfo;

pub const DEFAULT_TEMPLATE: &str = "{name}";

/// Platzhalter, die [`render`] versteht.
pub const PLACEHOLDERS: &[&str] = &[
    "name", "artist", "albumartist", "album", "title", "track", "disc", "year",
];

/// Ersatzwerte für fehlende Tags, abhängig von der Sprache der Oberfläche.
#[derive(Debug, Clone)]
pub struct Fallbacks {
    pub artist: String,
    pub album: String,
}

impl Default for Fallbacks {
    fn default() -> Self {
        Self { artist: "Unbekannter Interpret".into(), album: "Unbekanntes Album".into() }
    }
}

/// Vollständiger Zielpfad: `base_dir` + gerenderte Vorlage + Endung.
pub fn output_path(base_dir: &Path, template: &str, input: &Path, info: &MediaInfo, ext: &str) -> PathBuf {
    output_path_with(base_dir, template, input, info, ext, &Fallbacks::default())
}

/// Wie [`output_path`], aber mit eigenen Ersatzwerten für fehlende Tags.
pub fn output_path_with(
    base_dir: &Path,
    template: &str,
    input: &Path,
    info: &MediaInfo,
    ext: &str,
    fallbacks: &Fallbacks,
) -> PathBuf {
    let mut path = base_dir.join(render_with(template, input, info, fallbacks)).into_os_string();
    path.push(".");
    path.push(ext);
    PathBuf::from(path)
}

/// Rendert die Vorlage zu einem relativen Pfad ohne Endung. `/` in der Vorlage
/// erzeugt Unterordner; Werte aus Tags können keine Ordner erzeugen oder
/// aus dem Zielordner ausbrechen.
pub fn render(template: &str, input: &Path, info: &MediaInfo) -> PathBuf {
    render_with(template, input, info, &Fallbacks::default())
}

/// Wie [`render`], aber mit eigenen Ersatzwerten für fehlende Tags.
pub fn render_with(template: &str, input: &Path, info: &MediaInfo, fallbacks: &Fallbacks) -> PathBuf {
    let name = input
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "audio".into());
    let tag = |key: &str| info.tags.get(key).map(|v| v.trim()).filter(|v| !v.is_empty());

    let value = |key: &str| -> String {
        match key.to_lowercase().as_str() {
            "name" => name.clone(),
            "artist" => tag("artist").or(tag("album_artist")).unwrap_or(&fallbacks.artist).into(),
            "albumartist" => tag("album_artist").or(tag("artist")).unwrap_or(&fallbacks.artist).into(),
            "album" => tag("album").unwrap_or(&fallbacks.album).into(),
            "title" => tag("title").map(str::to_string).unwrap_or_else(|| name.clone()),
            "track" => tag("track")
                .and_then(leading_number)
                .map(|n| format!("{n:02}"))
                .unwrap_or_else(|| "00".into()),
            "disc" => tag("disc")
                .and_then(leading_number)
                .map(|n| n.to_string())
                .unwrap_or_else(|| "1".into()),
            "year" => tag("date").or(tag("year")).map(|d| d.chars().take(4).collect()).unwrap_or_default(),
            other => format!("{{{other}}}"),
        }
    };

    let mut rendered = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        rendered.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('}') {
            Some(end) => {
                rendered.push_str(&value(&after[..end]).replace(['/', '\\'], "-"));
                rest = &after[end + 1..];
            }
            None => {
                rendered.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    rendered.push_str(rest);

    let mut path = PathBuf::new();
    for part in rendered.split(|c| c == '/' || c == '\\') {
        let clean = sanitize_component(part);
        if !clean.is_empty() && clean != "." && clean != ".." {
            path.push(clean);
        }
    }
    if path.as_os_str().is_empty() {
        path.push(sanitize_component(&name));
    }
    path
}

/// Hängt " (2)", " (3)" … an, bis der Pfad weder existiert noch vergeben ist.
pub fn unique_path(path: PathBuf, is_taken: impl Fn(&Path) -> bool) -> PathBuf {
    if !path.exists() && !is_taken(&path) {
        return path;
    }
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let ext = path.extension().map(|e| e.to_string_lossy().into_owned());
    let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
    for n in 2u32.. {
        let file = match &ext {
            Some(e) => format!("{stem} ({n}).{e}"),
            None => format!("{stem} ({n})"),
        };
        let candidate = parent.join(file);
        if !candidate.exists() && !is_taken(&candidate) {
            return candidate;
        }
    }
    unreachable!()
}

fn leading_number(s: &str) -> Option<u32> {
    s.split('/').next()?.trim().parse().ok()
}

/// Entfernt Zeichen, die unter Windows, macOS oder Linux in Dateinamen stören.
fn sanitize_component(part: &str) -> String {
    let replaced: String = part
        .chars()
        .map(|c| if c.is_control() || "<>:\"|?*".contains(c) { '_' } else { c })
        .collect();
    let trimmed = replaced.trim().trim_end_matches('.').trim_end().to_string();

    const RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let base = trimmed.split('.').next().unwrap_or_default().to_ascii_uppercase();
    if RESERVED.contains(&base.as_str()) {
        format!("_{trimmed}")
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn info(tags: &[(&str, &str)]) -> MediaInfo {
        MediaInfo {
            format_name: String::new(),
            duration_secs: None,
            codec: String::new(),
            sample_rate: None,
            channels: None,
            bit_rate: None,
            bits_per_sample: None,
            has_cover: false,
            tags: tags.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn renders_folders_from_tags() {
        let i = info(&[("artist", "AC/DC"), ("album", "Back in Black"), ("title", "Hells Bells"), ("track", "1/10")]);
        let p = render("{artist}/{album}/{track} - {title}", Path::new("x.flac"), &i);
        assert_eq!(p, PathBuf::from("AC-DC").join("Back in Black").join("01 - Hells Bells"));
    }

    #[test]
    fn falls_back_to_file_name() {
        let p = render("{name}", Path::new("/music/Mein Lied.v2.flac"), &info(&[]));
        assert_eq!(p, PathBuf::from("Mein Lied.v2"));
    }

    #[test]
    fn blocks_traversal_and_bad_chars() {
        let p = render("../{title}", Path::new("a.mp3"), &info(&[("title", "Wer? Wie: Was*")]));
        assert_eq!(p, PathBuf::from("Wer_ Wie_ Was_"));
    }

    #[test]
    fn uses_given_fallbacks() {
        let fb = Fallbacks { artist: "Unknown Artist".into(), album: "Unknown Album".into() };
        let p = render_with("{artist}/{album}/{name}", Path::new("a.mp3"), &info(&[]), &fb);
        assert_eq!(p, PathBuf::from("Unknown Artist").join("Unknown Album").join("a"));
    }

    #[test]
    fn keeps_dotted_stem_when_adding_extension() {
        let p = output_path(Path::new("out"), "{name}", Path::new("Song.v2.flac"), &info(&[]), "mp3");
        assert_eq!(p, PathBuf::from("out").join("Song.v2.mp3"));
    }
}
