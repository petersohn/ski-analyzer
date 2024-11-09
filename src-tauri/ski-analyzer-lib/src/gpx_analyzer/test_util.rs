use super::AnalyzedRoute;

use std::fs::OpenOptions;

pub fn save_analyzed_route(piste: &AnalyzedRoute, filename: &str) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)
        .unwrap();
    serde_json::to_writer_pretty(file, &piste).unwrap();
}
