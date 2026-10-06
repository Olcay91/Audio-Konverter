//! Konvertierung ohne Oberfläche testen:
//!
//!   cargo run -p audioconv-core --example convert -- <eingabe> <preset-id> [ausgabeordner]

use std::io::Write;
use std::path::PathBuf;

use audioconv_core::{convert, naming, preset, probe, CancellationToken, Job, Tools};

#[tokio::main]
async fn main() -> audioconv_core::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let presets = preset::default_presets();

    if args.len() < 2 {
        eprintln!("Aufruf: convert <eingabe> <preset-id> [ausgabeordner]\n\nPresets:");
        for p in &presets {
            eprintln!("  {:<12} {}", p.id, p.name);
        }
        std::process::exit(2);
    }

    let tools = Tools::locate(&[]);
    println!("{}", tools.ffmpeg_version().await?);

    let input = PathBuf::from(&args[0]);
    let Some(preset) = presets.iter().find(|p| p.id == args[1]) else {
        eprintln!("Unbekanntes Preset: {}", args[1]);
        std::process::exit(2);
    };

    let info = probe(&tools, &input).await?;
    println!("{info:#?}");

    let out_dir = args
        .get(2)
        .map(PathBuf::from)
        .or_else(|| input.parent().map(PathBuf::from))
        .unwrap_or_default();
    let target = naming::output_path(&out_dir, naming::DEFAULT_TEMPLATE, &input, &info, preset.codec.extension());
    let output = naming::unique_path(target, |_| false);

    let cancel = CancellationToken::new();
    let job = Job { input: &input, output: &output, preset, info: &info };
    convert(&tools, job, &cancel, |f| {
        print!("\r{:5.1} %", f * 100.0);
        let _ = std::io::stdout().flush();
    })
    .await?;

    println!("\nFertig: {}", output.display());
    Ok(())
}
