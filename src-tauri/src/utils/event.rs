use serde::Serialize;
use tauri::{Emitter, Runtime};

pub fn emit_event<T: Serialize + Clone, R: Runtime>(
    emitter: &impl Emitter<R>,
    name: &str,
    data: &T,
) {
    if let Err(err) = emitter.emit(name, data) {
        eprintln!("Failed to send event {}: {}", name, err);
    }
}
