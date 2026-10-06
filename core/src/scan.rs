//! Aufklappen von Ordnern in einzelne Audiodateien.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Endungen, die beim Durchsuchen von Ordnern berücksichtigt werden. Einzeln
/// hinzugefügte Dateien werden unabhängig davon von ffprobe geprüft.
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "aac", "ac3", "aif", "aifc", "aiff", "alac", "amr", "ape", "au", "caf", "dff", "dsf", "dts",
    "flac", "m4a", "m4b", "mka", "mp2", "mp3", "mpc", "oga", "ogg", "opus", "tta", "w64", "wav",
    "wma", "wv",
];

pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Eine gefundene Datei samt Unterordner relativ zum hinzugefügten Ordner.
#[derive(Debug, Clone, PartialEq)]
pub struct ScannedFile {
    pub path: PathBuf,
    /// Leer bei einzeln hinzugefügten Dateien und bei Dateien direkt im Ordner.
    pub rel_dir: PathBuf,
}

/// Dateien bleiben, Ordner werden rekursiv nach Audiodateien durchsucht.
/// Reihenfolge bleibt erhalten, Duplikate fallen weg.
pub fn expand_paths(paths: &[PathBuf]) -> Vec<ScannedFile> {
    expand_paths_with(paths, |_| {})
}

/// Wie [`expand_paths`], meldet aber nach jeder gefundenen Datei die bisherige
/// Anzahl, damit eine Oberfläche bei großen Ordnern Fortschritt zeigen kann.
/// `on_found` wird oft aufgerufen; Drosseln ist Sache des Aufrufers.
pub fn expand_paths_with(paths: &[PathBuf], mut on_found: impl FnMut(usize)) -> Vec<ScannedFile> {
    let mut count = 0usize;
    let mut tick = || {
        count += 1;
        on_found(count);
    };
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for root in paths {
        if root.is_dir() {
            let mut found = Vec::new();
            walk(root, &mut found, &mut tick);
            found.sort();
            for path in found {
                if seen.insert(path.clone()) {
                    let rel_dir = path
                        .parent()
                        .and_then(|dir| dir.strip_prefix(root).ok())
                        .map(Path::to_path_buf)
                        .unwrap_or_default();
                    out.push(ScannedFile { path, rel_dir });
                }
            }
        } else if root.is_file() && seen.insert(root.clone()) {
            out.push(ScannedFile { path: root.clone(), rel_dir: PathBuf::new() });
            tick();
        }
    }
    out
}

fn walk(dir: &Path, found: &mut Vec<PathBuf>, tick: &mut dyn FnMut()) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else { continue };
        let path = entry.path();
        let hidden = entry.file_name().to_string_lossy().starts_with('.');
        // Symlinks auf Ordner werden nicht verfolgt (keine Endlosschleifen).
        if kind.is_dir() && !hidden {
            walk(&path, found, tick);
        } else if !kind.is_dir() && !hidden && is_audio_file(&path) {
            found.push(path);
            tick();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_relative_subfolders() {
        let root = std::env::temp_dir().join(format!("audioconv-scan-{}", std::process::id()));
        let album = root.join("Interpret").join("Album");
        std::fs::create_dir_all(&album).unwrap();
        std::fs::write(album.join("01.flac"), b"").unwrap();
        std::fs::write(root.join("lose.mp3"), b"").unwrap();
        std::fs::write(root.join("cover.jpg"), b"").unwrap();

        let found = expand_paths(&[root.clone()]);
        std::fs::remove_dir_all(&root).unwrap();

        assert_eq!(found.len(), 2, "cover.jpg darf nicht auftauchen");
        let flac = found.iter().find(|f| f.path.ends_with("01.flac")).unwrap();
        assert_eq!(flac.rel_dir, PathBuf::from("Interpret").join("Album"));
        let mp3 = found.iter().find(|f| f.path.ends_with("lose.mp3")).unwrap();
        assert_eq!(mp3.rel_dir, PathBuf::new());
    }
}
