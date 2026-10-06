//! Kern des Audio-Konverters, unabhängig von der Oberfläche.
//!
//! Ablauf eines Jobs: [`probe`] liest die Quelldatei aus, [`command::build_args`]
//! übersetzt ein [`Preset`] in ffmpeg-Argumente, [`convert`] führt ffmpeg aus und
//! meldet den Fortschritt.

pub mod command;
pub mod error;
pub mod naming;
pub mod preset;
pub mod probe;
pub mod progress;
pub mod runner;
pub mod scan;
pub mod tools;

pub use error::{Error, Result};
pub use preset::{Codec, Preset, PresetError, Rate};
pub use probe::{probe, MediaInfo};
pub use runner::{convert, Job};
pub use tools::Tools;
pub use tokio_util::sync::CancellationToken;
