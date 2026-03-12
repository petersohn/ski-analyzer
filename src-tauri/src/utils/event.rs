use serde_json::Value;
use tauri::{AppHandle, Emitter};

pub trait EventEmitter: Send + Sync {
    fn emit_event(&self, name: &str, data: &Value);
}

pub struct TauriEventEmitter(pub AppHandle);

impl EventEmitter for TauriEventEmitter {
    fn emit_event(&self, name: &str, data: &Value) {
        if let Err(err) = self.0.emit(name, data) {
            eprintln!("Failed to send event {}: {}", name, err);
        }
    }
}

#[cfg(test)]
pub mod test_helpers {
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    use crate::utils::event::EventEmitter;

    #[derive(Clone, Default)]
    pub struct MockEventEmitter {
        events: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
    }

    impl MockEventEmitter {
        pub fn new() -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub fn get_events(&self, name: Option<&str>) -> Vec<(String, String)> {
            let events = self.events.lock().unwrap();
            events
                .iter()
                .filter(|(n, _)| match name {
                    None => true,
                    Some(nn) => n == nn,
                })
                .map(|(n, data)| {
                    (n.clone(), String::from_utf8_lossy(data).to_string())
                })
                .collect()
        }

        pub fn clear(&self) {
            let mut events = self.events.lock().unwrap();
            events.clear();
        }
    }

    impl EventEmitter for MockEventEmitter {
        fn emit_event(&self, name: &str, data: &Value) {
            let json = serde_json::to_vec(data).unwrap_or_default();
            let mut events = self.events.lock().unwrap();
            events.push((name.to_string(), json));
        }
    }
}
