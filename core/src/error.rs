/// Fehler des Kerns. Die `Display`-Texte sind deutsch (CLI, Logs); Oberflächen
/// übersetzen anhand von [`Error::code`] und [`Error::detail`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{tool} wurde nicht gefunden oder ist nicht ausführbar ({source})")]
    ToolMissing {
        tool: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("Dateizugriff fehlgeschlagen: {0}")]
    Io(#[from] std::io::Error),
    #[error("ffprobe-Ausgabe ist unlesbar: {0}")]
    ProbeParse(#[from] serde_json::Error),
    #[error("Datei konnte nicht gelesen werden: {0}")]
    ProbeFailed(String),
    #[error("Datei konnte nicht gelesen werden: unbekanntes Format")]
    UnknownFormat,
    #[error("Datei enthält keine Audiospur")]
    NoAudio,
    #[error("Preset-Datei ist ungültig: {0}")]
    Preset(#[from] toml::de::Error),
    #[error("Eigene Presets konnten nicht gelesen oder gespeichert werden: {0}")]
    UserPresets(String),
    #[error("{0}")]
    FfmpegFailed(String),
    #[error("ffmpeg wurde mit Code {0:?} beendet")]
    FfmpegExit(Option<i32>),
    #[error("Abgebrochen")]
    Cancelled,
}

impl Error {
    /// Stabiler Code für Oberflächen, die den Text selbst übersetzen.
    pub fn code(&self) -> &'static str {
        match self {
            Error::ToolMissing { .. } => "toolMissing",
            Error::Io(_) => "io",
            Error::ProbeParse(_) => "probeParse",
            Error::ProbeFailed(_) => "probeFailed",
            Error::UnknownFormat => "unknownFormat",
            Error::NoAudio => "noAudio",
            Error::Preset(_) => "presetFile",
            Error::UserPresets(_) => "userPresets",
            Error::FfmpegFailed(_) => "ffmpegFailed",
            Error::FfmpegExit(_) => "ffmpegExit",
            Error::Cancelled => "cancelled",
        }
    }

    /// Sprachunabhängiger Zusatz (Programmname, Meldung des Systems oder von ffmpeg).
    pub fn detail(&self) -> Option<String> {
        match self {
            Error::ToolMissing { tool, .. } => Some((*tool).to_string()),
            Error::Io(e) => Some(e.to_string()),
            Error::ProbeParse(e) => Some(e.to_string()),
            Error::ProbeFailed(msg) | Error::UserPresets(msg) | Error::FfmpegFailed(msg) => Some(msg.clone()),
            Error::Preset(e) => Some(e.to_string()),
            Error::FfmpegExit(code) => Some(code.map_or_else(|| "?".to_string(), |c| c.to_string())),
            Error::UnknownFormat | Error::NoAudio | Error::Cancelled => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
