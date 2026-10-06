use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use audioconv_core::command::SkipReason;
use audioconv_core::{naming, CancellationToken, Error as CoreError, Job, MediaInfo, Preset, PresetError};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::state::{AppState, Item, Shared};

// ---------- Fehler für die Oberfläche ----------

/// Fehler mit stabilem Code; die Oberfläche übersetzt ihn (ui/src/lib/i18n).
/// `detail` ist sprachunabhängig, z. B. eine Meldung des Systems oder von ffmpeg.
#[derive(Debug, Clone, Serialize)]
pub struct UiError {
    code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

impl UiError {
    fn new(code: &'static str) -> Self {
        Self { code, detail: None }
    }

    fn with(code: &'static str, detail: impl ToString) -> Self {
        Self { code, detail: Some(detail.to_string()) }
    }
}

impl From<&CoreError> for UiError {
    fn from(e: &CoreError) -> Self {
        Self { code: e.code(), detail: e.detail() }
    }
}

impl From<CoreError> for UiError {
    fn from(e: CoreError) -> Self {
        (&e).into()
    }
}

impl From<PresetError> for UiError {
    fn from(e: PresetError) -> Self {
        Self::new(e.code())
    }
}

/// Sprache der Oberfläche, für Texte, die im Backend entstehen (Benachrichtigungen).
#[derive(Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    De,
    En,
}

// ---------- Umgebung & Presets ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    app_version: String,
    ffmpeg_path: String,
    ffmpeg_version: Option<String>,
    ffmpeg_error: Option<UiError>,
    parallel_jobs: usize,
}

#[tauri::command]
pub async fn environment(app: AppHandle, state: State<'_, AppState>) -> Result<Environment, UiError> {
    let s = state.0.clone();
    let (ffmpeg_version, ffmpeg_error) = match s.tools.ffmpeg_version().await {
        Ok(v) => (Some(v), None),
        Err(e) => (None, Some(UiError::from(e))),
    };
    Ok(Environment {
        app_version: app.package_info().version.to_string(),
        ffmpeg_path: s.tools.ffmpeg.display().to_string(),
        ffmpeg_version,
        ffmpeg_error,
        parallel_jobs: s.parallel,
    })
}

#[tauri::command]
pub fn presets(state: State<'_, AppState>) -> Vec<Preset> {
    state.0.presets.lock().unwrap().clone()
}

#[derive(Serialize)]
pub struct SavedPreset {
    id: String,
    presets: Vec<Preset>,
}

/// Neue ID für ein eigenes Preset. `n` unterscheidet mehrere IDs derselben Millisekunde.
fn new_preset_id(n: usize) -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    if n == 0 {
        format!("eigen-{millis}")
    } else {
        format!("eigen-{millis}-{n}")
    }
}

fn same_name(a: &Preset, b: &Preset) -> bool {
    a.codec == b.codec && a.name.trim().eq_ignore_ascii_case(b.name.trim())
}

/// Legt ein eigenes Preset an (leere `id`) oder ändert ein vorhandenes eigenes Preset.
#[tauri::command]
pub fn save_preset(state: State<'_, AppState>, preset: Preset) -> Result<SavedPreset, UiError> {
    let s = &state.0;
    let mut preset = preset;
    preset.name = preset.name.trim().to_string();
    preset.builtin = false;
    preset.validate()?;

    let mut presets = s.presets.lock().unwrap();
    if preset.id.is_empty() {
        preset.id = new_preset_id(0);
    } else if presets.iter().any(|p| p.id == preset.id && p.builtin) {
        return Err(UiError::new("presetBuiltinReadonly"));
    }
    if presets.iter().any(|p| p.id != preset.id && same_name(p, &preset)) {
        return Err(UiError::new("presetDuplicateName"));
    }

    let mut updated = presets.clone();
    match updated.iter_mut().find(|p| p.id == preset.id) {
        Some(existing) => *existing = preset.clone(),
        None => updated.push(preset.clone()),
    }
    persist_user_presets(s, &updated)?;
    *presets = updated;
    Ok(SavedPreset { id: preset.id, presets: presets.clone() })
}

#[tauri::command]
pub fn delete_preset(state: State<'_, AppState>, id: String) -> Result<Vec<Preset>, UiError> {
    let s = &state.0;
    let mut presets = s.presets.lock().unwrap();
    if presets.iter().any(|p| p.id == id && p.builtin) {
        return Err(UiError::new("presetBuiltinReadonly"));
    }
    let updated: Vec<Preset> = presets.iter().filter(|p| p.id != id).cloned().collect();
    persist_user_presets(s, &updated)?;
    *presets = updated;
    Ok(presets.clone())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    presets: Vec<Preset>,
    added: usize,
    replaced: usize,
    skipped: usize,
    invalid: usize,
    /// ID aus der Datei → ID in der App, damit die Preset-Auswahl je Format passt
    id_map: HashMap<String, String>,
}

/// Übernimmt eigene Presets aus einer Einstellungsdatei. Bei gleichem Namen im
/// selben Format wird das vorhandene Preset ersetzt (`replace`) oder das
/// importierte übersprungen. Standard-Presets werden nie ersetzt.
#[tauri::command]
pub fn import_presets(
    state: State<'_, AppState>,
    presets: Vec<serde_json::Value>,
    replace: bool,
) -> Result<ImportResult, UiError> {
    let s = &state.0;
    let mut current = s.presets.lock().unwrap();
    let mut updated = current.clone();
    let mut result = ImportResult {
        presets: Vec::new(),
        added: 0,
        replaced: 0,
        skipped: 0,
        invalid: 0,
        id_map: HashMap::new(),
    };

    for (n, value) in presets.into_iter().enumerate() {
        // Einzeln einlesen, damit ein kaputter Eintrag nicht den ganzen Import verhindert.
        let Ok(mut preset) = serde_json::from_value::<Preset>(value) else {
            result.invalid += 1;
            continue;
        };
        let old_id = preset.id.clone();
        preset.name = preset.name.trim().to_string();
        preset.builtin = false;
        if preset.validate().is_err() {
            result.invalid += 1;
            continue;
        }

        let new_id = match updated.iter().position(|p| same_name(p, &preset)) {
            Some(i) if updated[i].builtin || !replace => {
                result.skipped += 1;
                updated[i].id.clone()
            }
            Some(i) => {
                preset.id = updated[i].id.clone();
                updated[i] = preset;
                result.replaced += 1;
                updated[i].id.clone()
            }
            None => {
                if preset.id.is_empty() || updated.iter().any(|p| p.id == preset.id) {
                    preset.id = new_preset_id(n + 1);
                }
                let id = preset.id.clone();
                updated.push(preset);
                result.added += 1;
                id
            }
        };
        if !old_id.is_empty() {
            result.id_map.insert(old_id, new_id);
        }
    }

    if result.added + result.replaced > 0 {
        persist_user_presets(s, &updated)?;
        *current = updated;
    }
    result.presets = current.clone();
    Ok(result)
}

fn persist_user_presets(s: &Shared, all: &[Preset]) -> Result<(), UiError> {
    let path = s
        .user_presets_path
        .as_ref()
        .ok_or_else(|| UiError::new("presetStorageMissing"))?;
    let own: Vec<Preset> = all.iter().filter(|p| !p.builtin).cloned().collect();
    audioconv_core::preset::save_user_presets(path, &own).map_err(UiError::from)
}

// ---------- Einstellungsdateien (Export/Import) ----------

/// Einstellungsdateien sind klein; größere Dateien sind sicher keine.
const MAX_SETTINGS_FILE: u64 = 1024 * 1024;

/// Liest eine vom Nutzer gewählte Einstellungsdatei.
#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, UiError> {
    let len = std::fs::metadata(&path).map_err(|e| UiError::with("io", e))?.len();
    if len > MAX_SETTINGS_FILE {
        return Err(UiError::new("fileTooLarge"));
    }
    std::fs::read_to_string(&path).map_err(|e| UiError::with("io", e))
}

/// Schreibt eine Einstellungsdatei an den im Speichern-Dialog gewählten Ort.
#[tauri::command]
pub fn write_text_file(path: String, contents: String) -> Result<(), UiError> {
    std::fs::write(&path, contents).map_err(|e| UiError::with("io", e))
}

// ---------- Warteschlange ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddedItem {
    id: u64,
    path: String,
    file_name: String,
    info: Option<MediaInfo>,
    error: Option<UiError>,
}

/// Fortschritt beim Hinzufügen, als Event `add-progress`.
/// Phase `scanning`: Ordner werden durchsucht (`found` = bisher gefundene Audiodateien).
/// Phase `reading`: ffprobe liest die Dateien (`done` von `total`).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProgress {
    request_id: u64,
    phase: &'static str,
    found: usize,
    done: usize,
    total: usize,
}

/// Höchstens so oft ein Fortschritts-Event, damit die Oberfläche nicht überflutet wird.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(120);

#[derive(Serialize)]
pub struct AddResult {
    items: Vec<AddedItem>,
    /// Dateien, die schon in der Warteschlange stehen und übersprungen wurden
    skipped: usize,
}

/// Klappt Ordner auf, analysiert alle Dateien parallel und merkt sich die
/// konvertierbaren. Dateien ohne Audiospur kommen mit `error` zurück.
/// Mit `request_id` gibt es unterwegs `add-progress`-Events.
#[tauri::command]
pub async fn add_paths(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
    request_id: Option<u64>,
) -> Result<AddResult, UiError> {
    let s = state.0.clone();
    let progress = move |phase: &'static str, found: usize, done: usize, total: usize| {
        if let Some(request_id) = request_id {
            let _ = app.emit("add-progress", AddProgress { request_id, phase, found, done, total });
        }
    };
    progress("scanning", 0, 0, 0);

    let inputs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let scan_progress = progress.clone();
    let files = tauri::async_runtime::spawn_blocking(move || {
        let mut last = Instant::now();
        audioconv_core::scan::expand_paths_with(&inputs, |found| {
            if last.elapsed() >= PROGRESS_INTERVAL {
                last = Instant::now();
                scan_progress("scanning", found, 0, 0);
            }
        })
    })
    .await
    .map_err(|e| UiError::with("internal", e))?;

    let known: HashSet<PathBuf> = s.items.lock().unwrap().values().map(|i| i.path.clone()).collect();
    let (files, already): (Vec<_>, Vec<_>) = files.into_iter().partition(|f| !known.contains(&f.path));
    let total = files.len();
    progress("reading", total, 0, total);

    let probe_limit = Arc::new(Semaphore::new(8));
    let mut set = JoinSet::new();
    for (index, file) in files.into_iter().enumerate() {
        let s = s.clone();
        let limit = probe_limit.clone();
        set.spawn(async move {
            let _permit = limit.acquire_owned().await;
            let result = audioconv_core::probe(&s.tools, &file.path).await;
            (index, file, result)
        });
    }

    let mut results = Vec::new();
    let mut last = Instant::now();
    while let Some(joined) = set.join_next().await {
        if let Ok(r) = joined {
            results.push(r);
        }
        let done = results.len();
        if last.elapsed() >= PROGRESS_INTERVAL || done == total {
            last = Instant::now();
            progress("reading", total, done, total);
        }
    }
    results.sort_by_key(|(index, ..)| *index);

    let mut items = s.items.lock().unwrap();
    let added = results
        .into_iter()
        .map(|(_, file, result)| {
            let id = s.next_id();
            let audioconv_core::scan::ScannedFile { path, rel_dir } = file;
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let display = path.display().to_string();
            match result {
                Ok(info) => {
                    items.insert(id, Item { path, rel_dir, info: info.clone() });
                    AddedItem { id, path: display, file_name, info: Some(info), error: None }
                }
                Err(e) => AddedItem { id, path: display, file_name, info: None, error: Some(e.into()) },
            }
        })
        .collect();
    Ok(AddResult { items: added, skipped: already.len() })
}

#[tauri::command]
pub fn remove_items(state: State<'_, AppState>, ids: Vec<u64>) {
    let mut items = state.0.items.lock().unwrap();
    let running = state.0.running.lock().unwrap();
    for id in ids {
        if !running.contains_key(&id) {
            items.remove(&id);
        }
    }
}

// ---------- Konvertierung ----------

#[derive(Deserialize)]
pub struct NameFallbacks {
    artist: String,
    album: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRequest {
    ids: Vec<u64>,
    preset_id: String,
    /// `None` = neben die Originaldatei
    output_dir: Option<String>,
    template: Option<String>,
    /// Vorhandene Zieldateien ersetzen statt " (2)" anzuhängen
    #[serde(default)]
    overwrite: bool,
    /// Eigener Zielordner: Unterordner hinzugefügter Ordner nachbilden.
    /// Ordner der Originaldatei: Unterordner mit dem Formatnamen anlegen.
    #[serde(default)]
    subfolders: bool,
    /// Sprache der Systembenachrichtigung
    #[serde(default)]
    lang: Lang,
    /// Ersatz für fehlende Tags in der Dateinamen-Vorlage, in der Sprache der Oberfläche
    #[serde(default)]
    fallbacks: Option<NameFallbacks>,
}

#[derive(Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running,
    Done,
    Error,
    Cancelled,
    Skipped,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobUpdate {
    id: u64,
    status: JobStatus,
    progress: f64,
    output: Option<String>,
    error: Option<UiError>,
    /// Warum eine Datei übersprungen wurde
    skip: Option<SkipReason>,
}

impl JobUpdate {
    fn new(id: u64, status: JobStatus, progress: f64) -> Self {
        Self { id, status, progress, output: None, error: None, skip: None }
    }
}

#[derive(Clone, Serialize, Default)]
pub struct BatchSummary {
    done: usize,
    failed: usize,
    cancelled: usize,
    skipped: usize,
}

fn emit(app: &AppHandle, update: JobUpdate) {
    let _ = app.emit("job-update", update);
}

/// Titel und Text der Systembenachrichtigung nach einem Stapel.
fn notification_text(lang: Lang, done: usize, failed: usize) -> (&'static str, String) {
    match lang {
        Lang::De => match (done, failed) {
            (1, 0) => ("Konvertierung abgeschlossen", "1 Datei konvertiert".to_string()),
            (d, 0) => ("Konvertierung abgeschlossen", format!("{d} Dateien konvertiert")),
            (0, 1) => ("Konvertierung fehlgeschlagen", "Die Datei konnte nicht konvertiert werden.".to_string()),
            (0, f) => ("Konvertierung fehlgeschlagen", format!("Keine der {f} Dateien konnte konvertiert werden.")),
            (d, f) => ("Konvertierung abgeschlossen", format!("{d} konvertiert, {f} fehlgeschlagen")),
        },
        Lang::En => match (done, failed) {
            (1, 0) => ("Conversion complete", "1 file converted".to_string()),
            (d, 0) => ("Conversion complete", format!("{d} files converted")),
            (0, 1) => ("Conversion failed", "The file could not be converted.".to_string()),
            (0, f) => ("Conversion failed", format!("None of the {f} files could be converted.")),
            (d, f) => ("Conversion complete", format!("{d} converted, {f} failed")),
        },
    }
}

/// Startet die Jobs im Hintergrund und kehrt sofort zurück. Fortschritt kommt
/// als `job-update`-Event, das Ende des Stapels als `batch-finished`.
/// Gibt die Anzahl der gestarteten Jobs zurück (0 = nichts zu tun).
#[tauri::command]
pub fn start(app: AppHandle, state: State<'_, AppState>, request: StartRequest) -> Result<usize, UiError> {
    let s = state.0.clone();
    let preset = s
        .presets
        .lock()
        .unwrap()
        .iter()
        .find(|p| p.id == request.preset_id)
        .cloned()
        .ok_or_else(|| UiError::new("presetMissing"))?;
    let template = request
        .template
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| naming::DEFAULT_TEMPLATE.to_string());
    let output_dir = request.output_dir.filter(|d| !d.is_empty()).map(PathBuf::from);
    let (overwrite, subfolders, lang) = (request.overwrite, request.subfolders, request.lang);
    let fallbacks = request
        .fallbacks
        .map(|f| naming::Fallbacks { artist: f.artist, album: f.album })
        .unwrap_or_default();

    let mut jobs = Vec::new();
    {
        let items = s.items.lock().unwrap();
        let mut running = s.running.lock().unwrap();
        for id in request.ids {
            if running.contains_key(&id) {
                continue;
            }
            let Some(item) = items.get(&id) else { continue };
            let token = CancellationToken::new();
            running.insert(id, token.clone());
            jobs.push((id, item.path.clone(), item.rel_dir.clone(), item.info.clone(), token));
        }
    }
    if jobs.is_empty() {
        return Ok(0);
    }
    let started = jobs.len();
    for (id, ..) in &jobs {
        emit(&app, JobUpdate::new(*id, JobStatus::Queued, 0.0));
    }

    tauri::async_runtime::spawn(async move {
        let mut set = JoinSet::new();
        for (id, input, rel_dir, info, token) in jobs {
            let ctx = JobContext {
                app: app.clone(),
                shared: s.clone(),
                preset: preset.clone(),
                output_dir: output_dir.clone(),
                template: template.clone(),
                overwrite,
                subfolders,
                fallbacks: fallbacks.clone(),
            };
            set.spawn(run_job(ctx, id, input, rel_dir, info, token));
        }

        let mut summary = BatchSummary::default();
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok(JobStatus::Done) => summary.done += 1,
                Ok(JobStatus::Cancelled) => summary.cancelled += 1,
                Ok(JobStatus::Skipped) => summary.skipped += 1,
                _ => summary.failed += 1,
            }
        }

        if summary.done + summary.failed > 0 {
            let (title, body) = notification_text(lang, summary.done, summary.failed);
            let _ = app.notification().builder().title(title).body(body).show();
        }
        let _ = app.emit("batch-finished", summary);
    });

    Ok(started)
}

struct JobContext {
    app: AppHandle,
    shared: Arc<Shared>,
    preset: Preset,
    output_dir: Option<PathBuf>,
    template: String,
    overwrite: bool,
    subfolders: bool,
    fallbacks: naming::Fallbacks,
}

async fn run_job(
    ctx: JobContext,
    id: u64,
    input: PathBuf,
    rel_dir: PathBuf,
    info: MediaInfo,
    token: CancellationToken,
) -> JobStatus {
    let s = &ctx.shared;

    // Würde die Konvertierung nichts ändern oder nur verschlechtern? Dann gar nicht erst starten.
    if let Some(reason) = audioconv_core::command::skip_reason(&ctx.preset, &info) {
        s.running.lock().unwrap().remove(&id);
        let mut update = JobUpdate::new(id, JobStatus::Skipped, 0.0);
        update.skip = Some(reason);
        emit(&ctx.app, update);
        return JobStatus::Skipped;
    }

    // Auf einen freien Platz warten – oder auf einen Abbruch während des Wartens.
    let permit = tokio::select! {
        permit = s.limiter.clone().acquire_owned() => permit.ok(),
        _ = token.cancelled() => None,
    };
    let Some(_permit) = permit else {
        s.running.lock().unwrap().remove(&id);
        emit(&ctx.app, JobUpdate::new(id, JobStatus::Cancelled, 0.0));
        return JobStatus::Cancelled;
    };

    let source_dir = input.parent().map(Path::to_path_buf).unwrap_or_default();
    let base_dir = match &ctx.output_dir {
        // Eigener Zielordner: Struktur hinzugefügter Ordner nachbilden
        Some(dir) if ctx.subfolders => dir.join(&rel_dir),
        Some(dir) => dir.clone(),
        // Ordner der Originaldatei: Unterordner mit dem Formatnamen, z. B. "MP3"
        None if ctx.subfolders => source_dir.join(ctx.preset.codec.label()),
        None => source_dir,
    };
    let output = {
        let mut reserved = s.reserved.lock().unwrap();
        let target = naming::output_path_with(
            &base_dir,
            &ctx.template,
            &input,
            &info,
            ctx.preset.codec.extension(),
            &ctx.fallbacks,
        );
        // Überschreiben nur, wenn das Ziel weder die Quelle selbst ist noch
        // gerade von einem anderen Job desselben Stapels beschrieben wird.
        let output = if ctx.overwrite && target != input && !reserved.contains(&target) {
            target
        } else {
            naming::unique_path(target, |p| reserved.contains(p))
        };
        reserved.insert(output.clone());
        output
    };

    emit(&ctx.app, JobUpdate::new(id, JobStatus::Running, 0.0));
    let mut last_sent = 0.0_f64;
    let app = ctx.app.clone();
    let job = Job { input: &input, output: &output, preset: &ctx.preset, info: &info };
    let result = audioconv_core::convert(&s.tools, job, &token, |fraction| {
        // Höchstens alle 0,5 % ein Event, damit die Oberfläche nicht überflutet wird.
        if fraction - last_sent >= 0.005 || fraction >= 1.0 {
            last_sent = fraction;
            emit(&app, JobUpdate::new(id, JobStatus::Running, fraction));
        }
    })
    .await;

    s.reserved.lock().unwrap().remove(&output);
    s.running.lock().unwrap().remove(&id);

    let (status, update) = match result {
        Ok(()) => {
            let mut u = JobUpdate::new(id, JobStatus::Done, 1.0);
            u.output = Some(output.display().to_string());
            (JobStatus::Done, u)
        }
        Err(CoreError::Cancelled) => (JobStatus::Cancelled, JobUpdate::new(id, JobStatus::Cancelled, 0.0)),
        Err(e) => {
            let mut u = JobUpdate::new(id, JobStatus::Error, 0.0);
            u.error = Some(e.into());
            (JobStatus::Error, u)
        }
    };
    emit(&ctx.app, update);
    status
}

#[tauri::command]
pub fn cancel(state: State<'_, AppState>, id: u64) {
    if let Some(token) = state.0.running.lock().unwrap().get(&id) {
        token.cancel();
    }
}

#[tauri::command]
pub fn cancel_all(state: State<'_, AppState>) {
    for token in state.0.running.lock().unwrap().values() {
        token.cancel();
    }
}

// ---------- Installation ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallInfo {
    /// Kann der eingebaute Updater diese Installation ersetzen?
    can_self_update: bool,
}

/// Erkennt, ob ein Update direkt aus der App installiert werden kann:
/// - Windows: nur mit Installer (dann liegt `uninstall.exe` neben der App), nicht portabel
/// - macOS: ja (die .app wird ersetzt)
/// - Linux: nur als AppImage (`APPIMAGE` ist gesetzt), nicht als .deb
/// - Entwicklungsbuild: nie
#[tauri::command]
pub fn install_info() -> InstallInfo {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    let can_self_update = if cfg!(debug_assertions) {
        false
    } else if cfg!(target_os = "windows") {
        exe_dir.is_some_and(|dir| dir.join("uninstall.exe").is_file())
    } else if cfg!(target_os = "macos") {
        true
    } else {
        std::env::var_os("APPIMAGE").is_some()
    };
    InstallInfo { can_self_update }
}
