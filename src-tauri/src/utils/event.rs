use serde_json::Value;
use tauri::{AppHandle, Emitter};

pub trait EventEmitter: Send + Sync {
    fn emit_event(&self, name: &str, data: &Value);
}

pub struct TauriEventEmitter<'a>(pub &'a AppHandle);

impl<'a> EventEmitter for TauriEventEmitter<'a> {
    fn emit_event(&self, name: &str, data: &Value) {
        if let Err(err) = self.0.emit(name, data) {
            eprintln!("Failed to send event {}: {}", name, err);
        }
    }
}

#[cfg(test)]
pub mod test_helpers {
    use serde::Serialize;
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    use crate::utils::event::EventEmitter;

    pub trait TestEventEmitter: Send + Sync {
        fn emit_event<T: Serialize + Clone>(&self, name: &str, data: &T);
    }

    impl<T: EventEmitter> TestEventEmitter for T {
        fn emit_event<U: Serialize + Clone>(&self, name: &str, data: &U) {
            let value = serde_json::to_value(data).unwrap_or(Value::Null);
            EventEmitter::emit_event(self, name, &value);
        }
    }

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

        pub fn get_events(&self) -> Vec<(String, String)> {
            let events = self.events.lock().unwrap();
            events
                .iter()
                .map(|(name, data)| (name.clone(), String::from_utf8_lossy(data).to_string()))
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
