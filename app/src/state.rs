use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use audioconv_core::{CancellationToken, MediaInfo, Preset, Tools};
use tokio::sync::Semaphore;

/// Eine analysierte Datei in der Warteschlange.
pub struct Item {
    pub path: PathBuf,
    /// Unterordner relativ zum hinzugefügten Ordner (leer bei Einzeldateien)
    pub rel_dir: PathBuf,
    pub info: MediaInfo,
}

pub struct Shared {
    pub tools: Tools,
    /// Standard-Presets gefolgt von den eigenen
    pub presets: Mutex<Vec<Preset>>,
    /// Speicherort der eigenen Presets (presets.json im Konfigurationsordner)
    pub user_presets_path: Option<PathBuf>,
    /// Sperrreihenfolge immer: `items` vor `running`.
    pub items: Mutex<HashMap<u64, Item>>,
    pub running: Mutex<HashMap<u64, CancellationToken>>,
    /// Zielpfade laufender Jobs, damit parallele Jobs nicht in dieselbe Datei schreiben.
    pub reserved: Mutex<HashSet<PathBuf>>,
    pub limiter: Arc<Semaphore>,
    pub parallel: usize,
    next_id: AtomicU64,
}

impl Shared {
    pub fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

pub struct AppState(pub Arc<Shared>);

impl AppState {
    pub fn new(tools: Tools, user_presets_path: Option<PathBuf>) -> Self {
        let parallel = std::thread::available_parallelism()
            .map(|n| (n.get() / 2).max(1))
            .unwrap_or(2);
        let mut presets = audioconv_core::preset::default_presets();
        if let Some(path) = &user_presets_path {
            match audioconv_core::preset::load_user_presets(path) {
                Ok(own) => presets.extend(own),
                Err(e) => eprintln!("{e}"),
            }
        }
        Self(Arc::new(Shared {
            tools,
            presets: Mutex::new(presets),
            user_presets_path,
            items: Mutex::default(),
            running: Mutex::default(),
            reserved: Mutex::default(),
            limiter: Arc::new(Semaphore::new(parallel)),
            parallel,
            next_id: AtomicU64::new(1),
        }))
    }
}
