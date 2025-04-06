use ::function_name::named;
use geo::{LineString, Polygon};
use gpx::TrackSegment;
use std::collections::HashMap;
use std::fs;
use time::OffsetDateTime;

use super::find_pistes::find_pistes;
use super::{commit_moves, MoveType, Moving};
use crate::assert_eq_pretty;
use crate::gpx_analyzer::test_util::save_analyzed_route;
use crate::gpx_analyzer::{SegmentCoordinate, Segments};
use crate::ski_area::{Piste, SkiArea};
use crate::utils::cancel::CancellationToken;
use crate::utils::test_util::{
    create_ski_area_metadata, get_segments, line, make_gpx, piste, polygon,
    save_ski_area, segment,
};

fn ski_area(name: &str, pistes: &[Piste]) -> SkiArea {
    let piste_map = pistes
        .iter()
        .map(|p| (p.metadata.name.clone(), p.clone()))
        .collect();

    let mut result = SkiArea::new(
        create_ski_area_metadata(name.to_string()),
        HashMap::new(),
        piste_map,
        OffsetDateTime::now_utc(),
    )
    .unwrap();
    result.clip_piste_lines();
    result
}

fn save_activities(
    path: &str,
    segments: &Segments,
    data: &Vec<(Moving, SegmentCoordinate)>,
) {
    let result = commit_moves(&mut segments.clone(), data.clone());
    save_analyzed_route(&result, path);
}

fn run_test(
    name: &str,
    ski_area: SkiArea,
    track_segments: Vec<TrackSegment>,
    input: Vec<(MoveType, SegmentCoordinate)>,
    expected: Vec<(Moving, SegmentCoordinate)>,
) {
    let dir = format!("test_output/find_pistes_test/{name}");
    fs::create_dir_all(&dir).unwrap();
    save_ski_area(&ski_area, &format!("{dir}/ski_area.json"));

    let gpx = make_gpx(track_segments);
    let segments = get_segments(gpx);
    save_activities(&format!("{dir}/expected.json"), &segments, &expected);

    let actual =
        find_pistes(&CancellationToken::new(), &ski_area, &segments, input)
            .unwrap();
    save_activities(&format!("{dir}/actual.json"), &segments, &actual);
    assert_eq_pretty!(actual, expected);
}

// https://www.openstreetmap.org/way/232642673
fn line0() -> LineString {
    line(&[
        (6.5235995, 45.3108523),
        (6.5237814, 45.310974),
        (6.5246071, 45.3117733),
        (6.5263908, 45.3125859),
        (6.5265832, 45.3127541),
        (6.5266181, 45.3127846),
        (6.5267778, 45.3129252),
        (6.5271152, 45.3132564),
        (6.5271391, 45.3132798),
        (6.5274092, 45.3135446),
        (6.5274252, 45.3135603),
        (6.5278825, 45.3140085),
        (6.5280236, 45.3141538),
        (6.5282964, 45.314436),
        (6.5283769, 45.3144982),
        (6.5285058, 45.3145976),
        (6.5286941, 45.3147919),
        (6.5289989, 45.3151667),
        (6.5291222, 45.3154515),
        (6.5291758, 45.3155755),
        (6.5291763, 45.3155864),
        (6.5292266, 45.31671),
        (6.529334, 45.3175031),
        (6.5293445, 45.3175388),
        (6.5293971, 45.3177176),
        (6.5295696, 45.3183039),
        (6.5299086, 45.318929),
        (6.5301156, 45.3192459),
        (6.5303747, 45.3196734),
        (6.5308342, 45.3197677),
        (6.5313837, 45.3196566),
        (6.5334655, 45.3185371),
        (6.5340606, 45.3183577),
        (6.5344027, 45.3183701),
        (6.534727, 45.3184506),
        (6.5347477, 45.3184557),
        (6.5349588, 45.3185898),
        (6.5351054, 45.3187195),
        (6.5353989, 45.3189747),
    ])
}

// https://www.openstreetmap.org/way/200749479
fn area0() -> Polygon {
    polygon(&[
        (6.5235995, 45.3108523),
        (6.5239986, 45.3108322),
        (6.524421, 45.3112338),
        (6.5247703, 45.3115143),
        (6.5251762, 45.3118927),
        (6.525851, 45.3121598),
        (6.5262615, 45.3123417),
        (6.5263642, 45.3123872),
        (6.5264723, 45.312473),
        (6.5267024, 45.3126557),
        (6.5268272, 45.3127548),
        (6.5269955, 45.3128884),
        (6.5271885, 45.3130685),
        (6.5273013, 45.3131738),
        (6.5273849, 45.3132518),
        (6.5275881, 45.3134466),
        (6.5276364, 45.3134928),
        (6.5276549, 45.313516),
        (6.5278663, 45.3137811),
        (6.5281048, 45.3140166),
        (6.528162, 45.3140731),
        (6.5282108, 45.3141213),
        (6.5285194, 45.3143839),
        (6.5285813, 45.3144366),
        (6.5285968, 45.3144498),
        (6.5289541, 45.3147536),
        (6.5293491, 45.3153081),
        (6.5293613, 45.3153895),
        (6.5293812, 45.3155219),
        (6.5293875, 45.3156098),
        (6.5294237, 45.3161159),
        (6.5294072, 45.3167448),
        (6.5295185, 45.3172481),
        (6.5295453, 45.3173695),
        (6.5295771, 45.3175131),
        (6.5297682, 45.3182432),
        (6.5299688, 45.3186646),
        (6.5301977, 45.3191757),
        (6.5304714, 45.3195971),
        (6.530145, 45.3196528),
        (6.5297853, 45.3192637),
        (6.5295784, 45.3187872),
        (6.5293954, 45.3183859),
        (6.5293002, 45.318127),
        (6.5292639, 45.3180281),
        (6.5291405, 45.3176923),
        (6.5289683, 45.3167498),
        (6.52894, 45.3160811),
        (6.5289376, 45.3155152),
        (6.5286945, 45.3150722),
        (6.5283736, 45.3147934),
        (6.5281475, 45.3145836),
        (6.5278568, 45.3143138),
        (6.5272976, 45.3137048),
        (6.5265708, 45.3130128),
        (6.52642, 45.3128859),
        (6.5261866, 45.3126895),
        (6.5261271, 45.3126394),
        (6.5257142, 45.3123789),
        (6.5251242, 45.3121615),
        (6.5247372, 45.3120636),
        (6.5245083, 45.311969),
        (6.5241402, 45.3116753),
        (6.5237957, 45.3112355),
        (6.5236636, 45.3110397),
        (6.5236251, 45.3109271),
        (6.5235995, 45.3108523),
    ])
}

fn segments0() -> Vec<TrackSegment> {
    vec![segment(&[
        (6.5239132, 45.3109591),
        (6.5242407, 45.311227),
        (6.524431, 45.3117135),
        (6.5249571, 45.3120028),
        (6.5255282, 45.3122029),
        (6.5260645, 45.3123324),
        (6.5263981, 45.3127097),
        (6.5267808, 45.3130898),
    ])]
}

#[test]
#[named]
fn move_on_piste_with_line() {
    let ski_area =
        ski_area(function_name!(), &[piste("1", vec![line0()], vec![])]);
    let segments = segments0();
    let expected = vec![(
        Moving {
            piste_id: "1".to_string(),
            move_type: MoveType::Ski,
        },
        (0, 0),
    )];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}

#[test]
#[named]
fn move_on_piste_with_area() {
    let ski_area =
        ski_area(function_name!(), &[piste("1", vec![], vec![area0()])]);
    let segments = segments0();
    let expected = vec![(
        Moving {
            piste_id: "1".to_string(),
            move_type: MoveType::Ski,
        },
        (0, 0),
    )];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}
