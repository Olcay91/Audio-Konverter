use std::path::Path;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio_util::sync::CancellationToken;

use crate::command::build_args;
use crate::preset::Preset;
use crate::probe::MediaInfo;
use crate::progress::{ProgressEvent, ProgressParser};
use crate::tools::{command, Tools};
use crate::{Error, Result};

pub struct Job<'a> {
    pub input: &'a Path,
    pub output: &'a Path,
    pub preset: &'a Preset,
    pub info: &'a MediaInfo,
}

/// Führt eine Konvertierung aus. `on_progress` erhält Werte von 0.0 bis 1.0
/// (nur wenn die Dauer bekannt ist).
///
/// ffmpeg schreibt zuerst in eine versteckte Temp-Datei im Zielordner, die erst
/// nach Erfolg auf `job.output` umbenannt wird. Eine vorhandene Datei wird so nur
/// ersetzt, wenn die neue vollständig ist; bei Abbruch oder Fehler bleibt sie unberührt.
pub async fn convert(
    tools: &Tools,
    job: Job<'_>,
    cancel: &CancellationToken,
    mut on_progress: impl FnMut(f64),
) -> Result<()> {
    if let Some(parent) = job.output.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let temp = temp_path(job.output);
    let args = build_args(job.input, &temp, job.preset, job.info);
    let mut child = command(&tools.ffmpeg)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| Error::ToolMissing { tool: "ffmpeg", source })?;

    let stdout = child.stdout.take().expect("stdout ist gepipet");
    let mut stderr = child.stderr.take().expect("stderr ist gepipet");
    let stderr_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf).await;
        String::from_utf8_lossy(&buf).into_owned()
    });

    let parser = ProgressParser::new(job.info.duration_secs);
    let mut lines = BufReader::new(stdout).lines();

    let status = loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                let _ = child.kill().await;
                let _ = tokio::fs::remove_file(&temp).await;
                return Err(Error::Cancelled);
            }
            line = lines.next_line() => {
                match line? {
                    Some(line) => {
                        if let Some(ProgressEvent::Fraction(f)) = parser.feed(&line) {
                            on_progress(f);
                        }
                    }
                    None => break child.wait().await?,
                }
            }
        }
    };

    let stderr_text = stderr_task.await.unwrap_or_default();
    if status.success() {
        // rename ersetzt eine vorhandene Datei auf allen Plattformen.
        if let Err(e) = tokio::fs::rename(&temp, job.output).await {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(e.into());
        }
        on_progress(1.0);
        Ok(())
    } else {
        let _ = tokio::fs::remove_file(&temp).await;
        let msg = last_lines(&stderr_text, 4);
        Err(if msg.is_empty() { Error::FfmpegExit(status.code()) } else { Error::FfmpegFailed(msg) })
    }
}

/// `Ordner/Lied.mp3` -> `Ordner/.Lied.part.mp3` (Endung bleibt, damit ffmpeg das Format erkennt).
fn temp_path(output: &Path) -> std::path::PathBuf {
    let stem = output.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let name = match output.extension() {
        Some(ext) => format!(".{stem}.part.{}", ext.to_string_lossy()),
        None => format!(".{stem}.part"),
    };
    output.with_file_name(name)
}

fn last_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}
