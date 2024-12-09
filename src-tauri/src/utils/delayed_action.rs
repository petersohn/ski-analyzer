use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

type Func = Box<dyn Fn() + Send + Sync>;

pub struct DelayedAction {
    duration: Duration,
    state: Arc<Mutex<Option<Func>>>,
}

impl DelayedAction {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            state: Arc::new(Mutex::new(None)),
        }
    }

    pub fn call(&mut self, func: Func) {
        let func2 = {
            let mut lock = self.state.lock().unwrap();
            if lock.is_some() {
                *lock = None;
                Some(func)
            } else {
                *lock = Some(func);
                None
            }
        };

        if let Some(func) = func2 {
            self.state = Arc::new(Mutex::new(Some(func)));
        }

        let state = Arc::clone(&self.state);
        let duration = self.duration;
        tauri::async_runtime::spawn(async move {
            sleep(duration).await;
            let lock = state.lock().unwrap();
            if let Some(func) = &*lock {
                func();
            }
        });
    }
}
