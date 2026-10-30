//! Process-wide state. Cheap to clone (`Arc` inside) so background tasks and the protocol
//! handler can own a handle. Locks are std mutexes held only for short, non-async sections.

use std::ops::Deref;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use crate::core::models::{PresetCatalog, Settings};
use crate::core::preview::PreviewEncoder;
use crate::media::encoder::EncodeHandle;
use crate::media::session::Session;
use crate::media::tools::ToolPaths;
use crate::settings;

#[derive(Debug, Clone)]
pub struct Paths {
    /// Holds settings.json.
    pub config_dir: PathBuf,
    /// `<app_local_data_dir>/tools`: where downloads go.
    pub tools_dir: PathBuf,
    /// `tools/` next to the exe (portable layout), if the exe path is known.
    pub exe_tools_dir: Option<PathBuf>,
    /// `%TEMP%/videocrop`: parent of every session dir.
    pub temp_root: PathBuf,
}

/// Preview encoder chosen by hwdetect; None until detection finished (callers use libx264).
pub type SharedEncoder = Arc<RwLock<Option<PreviewEncoder>>>;

pub struct Inner {
    pub paths: Paths,
    pub settings: Mutex<Settings>,
    pub session: Mutex<Option<Arc<Session>>>,
    pub encode: Mutex<Option<EncodeHandle>>,
    /// Cached once both tools resolve; cleared after a (re)download.
    pub tools: RwLock<Option<ToolPaths>>,
    pub downloading: AtomicBool,
    pub preview_encoder: SharedEncoder,
    pub hwdetect_started: AtomicBool,
    catalog: OnceLock<PresetCatalog>,
}

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

impl Deref for AppState {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}

impl AppState {
    pub fn new(paths: Paths) -> Self {
        let settings = settings::load(&settings::file_path(&paths.config_dir));
        Self(Arc::new(Inner {
            paths,
            settings: Mutex::new(settings),
            session: Mutex::new(None),
            encode: Mutex::new(None),
            tools: RwLock::new(None),
            downloading: AtomicBool::new(false),
            preview_encoder: Arc::new(RwLock::new(None)),
            hwdetect_started: AtomicBool::new(false),
            catalog: OnceLock::new(),
        }))
    }

    pub fn catalog(&self) -> &PresetCatalog {
        self.catalog.get_or_init(crate::core::presets::catalog)
    }

    pub fn settings_snapshot(&self) -> Settings {
        lock(&self.settings).clone()
    }

    /// Mutates settings under the lock, then persists the result.
    pub fn update_settings(&self, f: impl FnOnce(&mut Settings)) -> Result<(), String> {
        let snapshot = {
            let mut s = lock(&self.settings);
            f(&mut s);
            s.clone()
        };
        settings::save(&settings::file_path(&self.paths.config_dir), &snapshot)
    }

    pub fn current_session(&self) -> Option<Arc<Session>> {
        lock(&self.session).clone()
    }

    pub fn session_by_id(&self, id: &str) -> Option<Arc<Session>> {
        lock(&self.session).as_ref().filter(|s| s.id == id).cloned()
    }

    pub fn take_session(&self) -> Option<Arc<Session>> {
        lock(&self.session).take()
    }
}

/// Lock that shrugs off poisoning (a panicked holder must not take the whole app down).
pub fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
