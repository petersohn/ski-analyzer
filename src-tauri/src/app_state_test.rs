use geo::coord;
use rstest::{fixture, rstest};
use ski_analyzer_lib::ski_area::{SkiArea, SkiAreaMetadata};
use ski_analyzer_lib::utils::bounded_geometry::BoundedGeometry;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::utils::event::test_helpers::MockEventEmitter;

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let temp_dir = std::env::temp_dir()
            .join(format!("ski_analyzer_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        Self(temp_dir)
    }

    fn path(&self) -> &PathBuf {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[fixture]
fn temp_dir() -> TempDir {
    TempDir::new()
}

#[fixture]
fn ski_area_a() -> SkiArea {
    create_ski_area("Area A".to_string())
}

#[fixture]
fn ski_area_b() -> SkiArea {
    create_ski_area("Area B".to_string())
}

fn create_ski_area(name: String) -> SkiArea {
    let bounding_rect =
        geo::Rect::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 1.0, y: 1.0 });
    SkiArea {
        metadata: SkiAreaMetadata {
            name: name.clone(),
            id: 0,
            outline: BoundedGeometry {
                item: geo::Polygon::new(
                    geo::LineString::new(vec![
                        coord! { x: 0.0, y: 0.0 },
                        coord! { x: 1.0, y: 0.0 },
                        coord! { x: 1.0, y: 1.0 },
                        coord! { x: 0.0, y: 1.0 },
                        coord! { x: 0.0, y: 0.0 },
                    ]),
                    vec![],
                ),
                bounding_rect,
            },
        },
        lifts: HashMap::new(),
        pistes: HashMap::new(),
        bounding_rect,
        date: time::OffsetDateTime::now_utc(),
    }
}

fn get_app_state(path: &PathBuf) -> AppState {
    let emitter = Arc::new(MockEventEmitter::new());
    let mut state = AppState::default();
    state.init_config(path, emitter);
    state
}

#[rstest]
fn test_cache_flow(
    temp_dir: TempDir,
    ski_area_a: SkiArea,
    ski_area_b: SkiArea,
) {
    let mut app_state = get_app_state(temp_dir.path());

    app_state.set_ski_area(ski_area_a.clone());
    let uuid_a = app_state.get_ski_area().unwrap().0;

    app_state.set_ski_area(ski_area_b.clone());
    let uuid_b = app_state.get_ski_area().unwrap().0;

    assert_ne!(uuid_a, uuid_b, "UUIDs should be different");

    let cached = app_state.get_cached_ski_areas();
    assert_eq!(cached.len(), 2, "Both ski areas should be cached");

    assert!(
        cached.contains_key(&uuid_a),
        "Ski area A should be in cache"
    );
    assert!(
        cached.contains_key(&uuid_b),
        "Ski area B should be in cache"
    );

    let current_uuid = app_state.get_config().current_ski_area;
    assert_eq!(current_uuid, Some(uuid_b), "Ski area B should be current");
    assert_eq!(
        app_state.get_ski_area().unwrap().1.metadata.name,
        "Area B",
        "Ski area B name should match"
    );
}

#[rstest]
fn test_cache_persistence_after_restart(
    temp_dir: TempDir,
    ski_area_a: SkiArea,
    ski_area_b: SkiArea,
) {
    let (uuid_a, uuid_b) = {
        let mut app_state = get_app_state(temp_dir.path());

        app_state.set_ski_area(ski_area_a.clone());
        let uuid_a = app_state.get_ski_area().unwrap().0;
        app_state.set_ski_area(ski_area_b.clone());
        let uuid_b = app_state.get_ski_area().unwrap().0;

        let cached = app_state.get_cached_ski_areas();
        assert_eq!(cached.len(), 2, "Both ski areas should be cached");
        (uuid_a, uuid_b)
    };

    {
        let app_state = get_app_state(temp_dir.path());

        let cached = app_state.get_cached_ski_areas();
        assert_eq!(cached.len(), 2, "Cache should persist after restart");

        assert!(
            cached.contains_key(&uuid_a),
            "Ski area A should be in cache"
        );
        assert!(
            cached.contains_key(&uuid_b),
            "Ski area B should be in cache"
        );

        let current_uuid = app_state.get_config().current_ski_area;
        assert_eq!(current_uuid, Some(uuid_b), "Ski area B should be current");
        assert_eq!(
            app_state.get_ski_area().unwrap().1.metadata.name,
            "Area B",
            "Ski area B name should match"
        );
    }
}

#[rstest]
fn test_load_cached_ski_area(
    temp_dir: TempDir,
    ski_area_a: SkiArea,
    ski_area_b: SkiArea,
) {
    let mut app_state = get_app_state(temp_dir.path());
    app_state.set_ski_area(ski_area_a.clone());
    let uuid_a = app_state.get_ski_area().unwrap().0;
    app_state.set_ski_area(ski_area_b.clone());
    let uuid_b = app_state.get_ski_area().unwrap().0;

    let current_uuid = app_state.get_config().current_ski_area;
    assert_eq!(current_uuid, Some(uuid_b), "Ski area B should be current");
    assert_eq!(
        app_state.get_ski_area().unwrap().1.metadata.name,
        "Area B",
        "Ski area B name should match"
    );

    app_state.load_cached_ski_area(&uuid_a).unwrap();
    let current_uuid = app_state.get_config().current_ski_area;
    assert_eq!(current_uuid, Some(uuid_a), "Ski area A should be current");
    assert_eq!(
        app_state.get_ski_area().unwrap().1.metadata.name,
        "Area A",
        "Ski area A name should match"
    );
}
