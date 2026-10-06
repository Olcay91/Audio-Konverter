use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use crate::{Error, Result};

/// Pfade zu ffmpeg und ffprobe.
#[derive(Debug, Clone)]
pub struct Tools {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
}

impl Tools {
    /// Sucht die Programme in dieser Reihenfolge: in `search_dirs` (z. B. neben der
    /// App, wohin Tauri mitgelieferte Sidecars legt), im `PATH`, in üblichen
    /// Installationsordnern. Wird nichts gefunden, bleibt der nackte Programmname,
    /// damit der Fehler erst beim Aufruf mit klarer Meldung auftritt.
    pub fn locate(search_dirs: &[PathBuf]) -> Self {
        Self {
            ffmpeg: find("ffmpeg", search_dirs),
            ffprobe: find("ffprobe", search_dirs),
        }
    }

    /// Erste Zeile von `ffmpeg -version`, z. B. "ffmpeg version 7.1 ...".
    pub async fn ffmpeg_version(&self) -> Result<String> {
        let out = command(&self.ffmpeg)
            .arg("-version")
            .output()
            .await
            .map_err(|source| Error::ToolMissing { tool: "ffmpeg", source })?;
        let text = String::from_utf8_lossy(&out.stdout);
        Ok(text.lines().next().unwrap_or_default().to_string())
    }
}

fn find(name: &str, search_dirs: &[PathBuf]) -> PathBuf {
    let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);

    let from_path = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
        .unwrap_or_default();

    // macOS-Apps aus dem Finder erben den PATH der Shell nicht.
    let fallback: Vec<PathBuf> = if cfg!(target_os = "macos") {
        vec!["/opt/homebrew/bin".into(), "/usr/local/bin".into(), "/opt/local/bin".into()]
    } else if cfg!(target_os = "linux") {
        vec!["/usr/bin".into(), "/usr/local/bin".into()]
    } else {
        Vec::new()
    };

    search_dirs
        .iter()
        .chain(from_path.iter())
        .chain(fallback.iter())
        .map(|dir| dir.join(&file))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| PathBuf::from(file))
}

/// Erstellt einen Prozess ohne Konsolenfenster (Windows) und ohne stdin.
pub(crate) fn command(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.stdin(Stdio::null()).kill_on_drop(true);
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}
