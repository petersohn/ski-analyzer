use std::collections::HashMap;
use std::fs::create_dir_all;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use ski_analyzer_lib::error::{Error, ErrorType, Result};
use ski_analyzer_lib::gpx_analyzer::AnalyzedRoute;
use ski_analyzer_lib::ski_area::SkiArea;
use ski_analyzer_lib::utils::cancel::{
    Cancellable, CancellableTask, CancellationToken,
};
use ski_analyzer_lib::utils::json::{
    load_from_file, load_from_file_if_exists, save_to_file,
};

use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, Position, Runtime, Size,
    Window, WindowEvent,
};
use uuid::Uuid;

use crate::config::{CachedSkiArea, Config, MapConfig, WindowConfig};
use crate::utils::delayed_action::DelayedAction;

pub struct AppState {
    config_path: PathBuf,
    config_file_path: PathBuf,
    ski_areas_path: PathBuf,
    config: Option<Config>,
    window_initialized: bool,
    window_saver: DelayedAction,
    ski_area: Option<Arc<(Uuid, SkiArea)>>,
    analyzed_route: Option<AnalyzedRoute>,
    task_id: u64,
    active_tasks: HashMap<u64, Arc<dyn Cancellable + Send + Sync>>,
}

fn remove_file(path: &Path) {
    if let Err(err) = std::fs::remove_file(path) {
        eprintln!("Failed to remove {:?}: {}", path, err);
    }
}

fn emit_event<T: Serialize + Clone, R: Runtime>(
    app_handle: &AppHandle<R>,
    name: &str,
    data: &T,
) {
    if let Err(err) = app_handle.emit(name, data) {
        eprintln!("Failed to send event {}: {}", name, err);
    }
}

impl AppState {
    pub fn init_config<M: Manager<R>, R: Runtime>(&mut self, manager: &M) {
        self.config_path = PathBuf::from(manager.path().data_dir().unwrap());
        self.config_path.push("ski-analyzer");
        self.config_file_path = self.config_path.join("config.json");
        self.ski_areas_path = self.config_path.join("ski_areas");
        let config = self.load_config().unwrap_or_else(|err| {
            eprintln!("Failed to load config: {}", err);
            Config::default()
        });

        if let Some(uuid) = config.current_ski_area {
            if let Err(err) =
                self.load_cached_ski_area_inner(manager.app_handle(), &uuid)
            {
                eprintln!("Failed to load ski area: {}", err);
            }
        }

        self.config = Some(config);
    }

    fn load_config(&self) -> Result<Config> {
        load_from_file(&self.config_file_path)
    }

    fn save_config_inner(
        &self,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        create_dir_all(&self.config_path)?;
        eprintln!("save config -> {:?}", self.config_file_path);
        Ok(save_to_file(self.get_config(), &self.config_file_path)?)
    }

    fn save_config(&self) {
        if let Err(err) = self.save_config_inner() {
            eprintln!("Failed to save config: {}", err);
        }
    }

    fn save_config_immediately(&mut self) {
        self.save_config();
        self.window_saver.cancel();
    }

    fn save_config_delayed<M: Manager<R>, R: Runtime>(&mut self, manager: &M) {
        let state = Arc::clone(manager.state::<AppStateType>().inner());
        self.window_saver.call(Box::new(move || {
            state.lock().unwrap().save_config();
        }));
    }

    pub fn get_config(&self) -> &Config {
        self.config.as_ref().unwrap()
    }

    fn get_config_mut(&mut self) -> &mut Config {
        self.config.as_mut().unwrap()
    }

    pub fn get_ski_area(&self) -> Option<&(Uuid, SkiArea)> {
        self.ski_area.as_ref().map(|s| &**s)
    }

    fn set_ski_area_inner<R: Runtime>(
        &mut self,
        app_handle: &AppHandle<R>,
        ski_area: SkiArea,
        uuid: Uuid,
    ) {
        self.ski_area = Some(Arc::new((uuid, ski_area)));
        emit_event(
            app_handle,
            "active_ski_area_changed",
            &self.ski_area.as_ref().unwrap().1,
        );
    }

    pub fn set_ski_area<R: Runtime>(
        &mut self,
        app_handle: &AppHandle<R>,
        ski_area: SkiArea,
    ) {
        let uuid = self.get_config_mut().save_ski_area(&ski_area);

        if let Err(err) = self.save_ski_area(&uuid, &ski_area) {
            eprintln!("Failed to save ski area: {}", err);
            self.get_config_mut().remove_ski_area(&uuid);
            return;
        }

        self.clear_route(app_handle);
        self.set_ski_area_inner(app_handle, ski_area, uuid);
    }

    fn save_ski_area(
        &mut self,
        uuid: &Uuid,
        ski_area: &SkiArea,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.window_saver.cancel();
        self.save_ski_area_inner(uuid, ski_area)?;
        self.get_config_mut().save_current_ski_area(Some(*uuid));
        self.save_config_immediately();

        Ok(())
    }

    fn save_ski_area_inner(
        &mut self,
        uuid: &Uuid,
        ski_area: &SkiArea,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        create_dir_all(&self.ski_areas_path)?;
        let path = self.get_ski_area_path(uuid);
        save_to_file(ski_area, &path)?;
        Ok(())
    }

    pub fn get_clipped_ski_area(&mut self) -> Option<(Uuid, SkiArea)> {
        let uuid = match self.ski_area.as_ref() {
            None => return None,
            Some(x) => x.0,
        };

        let cached_ski_area = self.get_config().ski_areas.get(&uuid).unwrap();
        if let Some(clipped_uuid) = cached_ski_area.clipped_uuid {
            let path = self.get_ski_area_path(&clipped_uuid);
            let result: Result<SkiArea> = load_from_file(&path);
            match result {
                Ok(ski_area) => return Some((uuid, ski_area)),
                Err(err) => {
                    eprintln!("Failed to read cached clipped ski area: {}", err)
                }
            };
        }

        let mut clipped = self.ski_area.as_ref().unwrap().1.clone();
        clipped.clip_piste_lines();
        let clipped_uuid = Uuid::new_v4();
        let path = self.get_ski_area_path(&clipped_uuid);
        if let Err(err) = save_to_file(&clipped, &path) {
            eprintln!("Failed to save clipped ski area: {}", err);
            return Some((uuid, clipped));
        }

        let cached_ski_area_mut =
            self.get_config_mut().ski_areas.get_mut(&uuid).unwrap();
        cached_ski_area_mut.clipped_uuid = Some(clipped_uuid);
        self.save_config_immediately();

        Some((uuid, clipped))
    }

    fn get_ski_area_path(&self, uuid: &Uuid) -> PathBuf {
        self.ski_areas_path.join(format!("{}.json", uuid))
    }

    fn load_cached_ski_area_inner<R: Runtime>(
        &mut self,
        app_handle: &AppHandle<R>,
        uuid: &Uuid,
    ) -> Result<()> {
        let result: SkiArea =
            match load_from_file_if_exists(&self.get_ski_area_path(uuid))? {
                None => {
                    self.remove_cached_ski_area(app_handle, uuid);
                    return Err(Error::new_s(
                        ErrorType::ExternalError,
                        "Ski area file not found",
                    ));
                }
                Some(s) => s,
            };
        self.set_ski_area_inner(app_handle, result.clone(), *uuid);
        Ok(())
    }

    pub fn load_cached_ski_area<R: Runtime>(
        &mut self,
        app_handle: &AppHandle<R>,
        uuid: &Uuid,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.load_cached_ski_area_inner(app_handle, uuid)?;
        if self.get_config_mut().save_current_ski_area(Some(*uuid)) {
            self.save_config_immediately();
        }

        Ok(())
    }

    pub fn get_cached_ski_areas(&self) -> &HashMap<Uuid, CachedSkiArea> {
        &self.get_config().ski_areas
    }

    pub fn remove_cached_ski_area<R: Runtime>(
        &mut self,
        app_handle: &AppHandle<R>,
        uuid: &Uuid,
    ) {
        let config = self.get_config_mut();
        let clipped_uuid = config
            .remove_ski_area(uuid)
            .map(|x| x.clipped_uuid)
            .flatten();
        let should_clear = config
            .current_ski_area
            .as_ref()
            .map_or(false, |saved| saved == uuid);

        if should_clear {
            config.current_ski_area = None;
            self.ski_area = None;
            emit_event(app_handle, "active_ski_area_changed", &self.ski_area);
        }

        remove_file(&self.get_ski_area_path(uuid));

        if let Some(clipped) = clipped_uuid {
            remove_file(&self.get_ski_area_path(&clipped));
        }
        self.save_config_immediately();
    }

    pub fn get_current_cached_ski_area(&self) -> Option<&CachedSkiArea> {
        let config = self.get_config();
        config.ski_areas.get(&config.current_ski_area?)
    }

    pub fn get_route(&self) -> Option<&AnalyzedRoute> {
        self.analyzed_route.as_ref()
    }

    pub fn set_route<R: Runtime>(
        &mut self,
        app_handle: &AppHandle<R>,
        route: AnalyzedRoute,
    ) {
        self.analyzed_route = Some(route);
        emit_event(app_handle, "active_route_changed", &self.analyzed_route);
    }

    pub fn clear_route<R: Runtime>(&mut self, app_handle: &AppHandle<R>) {
        self.analyzed_route = None;
        emit_event(app_handle, "active_route_changed", &self.analyzed_route);
    }

    pub fn save_map_config<M: Manager<R>, R: Runtime>(
        &mut self,
        manager: &M,
        config: MapConfig,
    ) {
        if self.get_config_mut().save_map_config(config) {
            self.save_config_delayed(manager);
        }
    }

    pub fn handle_window_event(
        &mut self,
        window: &Window,
        event: &WindowEvent,
    ) {
        let res = match event {
            WindowEvent::Resized(_) => self.save_window_config(window),
            _ => Ok(()),
        };
        if let Err(err) = res {
            eprintln!("Failed to save window position: {}", err);
            return;
        };
    }

    fn save_window_config(&mut self, window: &Window) -> tauri::Result<()> {
        if !self.window_initialized {
            self.window_initialized = true;
            if let Some(c) = &self.get_config().window_config {
                return self.init_window(window, c);
            }
        }

        if self.get_config_mut().save_window_config(window)? {
            self.save_config_delayed(window);
        }
        Ok(())
    }

    fn init_window(
        &self,
        window: &Window,
        config: &WindowConfig,
    ) -> tauri::Result<()> {
        window.set_size(Size::Physical(config.size))?;
        if let Some(monitor) = window.current_monitor()? {
            let ms = monitor.size();
            let mp = monitor.position();
            let ws = &config.size;
            let pos = PhysicalPosition::new(
                mp.x + (ms.width as i32 - ws.width as i32) / 2,
                mp.y + (ms.height as i32 - ws.height as i32) / 2,
            );
            window.set_position(Position::Physical(pos))?;
        }
        if config.maximized {
            window.maximize()?;
        }
        if config.fullscreen {
            window.set_fullscreen(true)?;
        }

        Ok(())
    }

    fn add_task(&mut self, cancel: Arc<dyn Cancellable + Send + Sync>) -> u64 {
        self.task_id += 1;
        self.active_tasks.insert(self.task_id, cancel);
        self.task_id
    }

    fn remove_task(&mut self, id: u64) {
        self.active_tasks.remove(&id);
    }

    pub fn add_sync_task<M, R, F, Ret>(manager: &M, func: F) -> Result<Ret>
    where
        M: Manager<R>,
        R: Runtime,
        F: FnOnce(&CancellationToken) -> Result<Ret>,
    {
        let state = manager.state::<AppStateType>();
        let cancel = Arc::new(CancellationToken::new());
        let task_id = state.lock().unwrap().add_task(cancel.clone());
        let ret = func(&*cancel);
        state.lock().unwrap().remove_task(task_id);
        ret
    }

    pub async fn add_async_task<M, R, F, Ret>(
        manager: &M,
        future: F,
    ) -> Result<Ret>
    where
        M: Manager<R>,
        R: Runtime,
        F: Future<Output = Result<Ret>> + Send + 'static,
        Ret: Send + 'static,
    {
        let state = manager.state::<AppStateType>();
        let (fut, cancel) = CancellableTask::spawn(future);
        let task_id = state.lock().unwrap().add_task(Arc::new(cancel));
        let ret = fut.await;
        state.lock().unwrap().remove_task(task_id);
        ret
    }

    pub fn cancel_all_tasks(&mut self) {
        for (_, task) in &self.active_tasks {
            task.cancel();
        }
        self.active_tasks.clear();
    }

    pub fn get_ui_config(&self) -> String {
        self.get_config().ui_config.clone()
    }

    pub fn set_ui_config(&mut self, config: String) {
        self.get_config_mut().ui_config = config;
        self.save_config_immediately();
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            config_path: PathBuf::new(),
            config_file_path: PathBuf::new(),
            ski_areas_path: PathBuf::new(),
            config: None,
            window_initialized: false,
            window_saver: DelayedAction::new(Duration::from_secs(2)),
            ski_area: None,
            analyzed_route: None,
            task_id: 0,
            active_tasks: HashMap::new(),
        }
    }
}

pub type AppStateType = Arc<Mutex<AppState>>;
