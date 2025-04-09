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

// https://www.openstreetmap.org/way/232642673
fn line1() -> LineString {
    line(&[
        (6.5235995, 45.3108523),
        (6.5238952, 45.3106622),
        (6.5240063, 45.3105181),
        (6.5242586, 45.3103239),
        (6.5246672, 45.3102071),
        (6.5266132, 45.3101133),
        (6.526996, 45.3100185),
        (6.5272932, 45.3099406),
        (6.5274993, 45.3099771),
        (6.5275277, 45.3099821),
        (6.5276563, 45.3101715),
        (6.5275421, 45.310505),
        (6.5273596, 45.3107711),
        (6.5269992, 45.3110565),
        (6.5267505, 45.3111595),
        (6.5263773, 45.3113368),
        (6.5258844, 45.3116325),
        (6.5258591, 45.3118367),
        (6.5259661, 45.3119306),
        (6.5261195, 45.3120652),
        (6.5264232, 45.3124814),
        (6.526347, 45.3127066),
        (6.5262003, 45.3128108),
        (6.5262352, 45.3128318),
        (6.5265832, 45.3127541),
        (6.5271188, 45.3125517),
        (6.5272994, 45.3125793),
        (6.5274604, 45.312739),
        (6.527331, 45.3130442),
        (6.5271391, 45.3132798),
        (6.5270372, 45.3133913),
        (6.5271252, 45.3135082),
        (6.5273077, 45.3135659),
        (6.5274252, 45.3135603),
        (6.527533, 45.3135528),
        (6.5284683, 45.3130917),
        (6.5286699, 45.3128523),
        (6.5288527, 45.3128624),
        (6.5288618, 45.3129475),
        (6.5287886, 45.3131232),
        (6.5284014, 45.3135513),
        (6.5280893, 45.3140598),
        (6.5280236, 45.3141538),
        (6.5279761, 45.3142334),
        (6.5278365, 45.3143629),
        (6.5276073, 45.3145483),
        (6.5276089, 45.3146358),
        (6.5276974, 45.3146805),
        (6.5279035, 45.3146678),
        (6.5283769, 45.3144982),
        (6.528524, 45.3144442),
        (6.5290926, 45.3142334),
        (6.5302243, 45.3138616),
        (6.5308868, 45.3139583),
        (6.5308826, 45.3140453),
        (6.5295678, 45.3152341),
        (6.5293955, 45.3153136),
        (6.5291971, 45.3154021),
        (6.5291222, 45.3154515),
        (6.5290957, 45.3154841),
        (6.5290832, 45.3155493),
        (6.5291174, 45.3155845),
        (6.5291763, 45.3155864),
        (6.5292228, 45.3155903),
        (6.5297584, 45.3155092),
        (6.5298611, 45.3155658),
        (6.5297522, 45.3159535),
        (6.5297969, 45.3159974),
        (6.5298778, 45.3160074),
        (6.529955, 45.3159726),
        (6.5302671, 45.3155777),
        (6.5303786, 45.3155653),
        (6.5304458, 45.3156249),
        (6.5303712, 45.3157689),
        (6.5303181, 45.3160918),
        (6.5301688, 45.3165922),
        (6.5293445, 45.3175388),
        (6.5292877, 45.31803),
        (6.5291402, 45.318191),
        (6.5285555, 45.3185901),
        (6.5285481, 45.3187468),
        (6.5286574, 45.3190237),
        (6.5286008, 45.3195),
        (6.5286502, 45.319779),
        (6.5281081, 45.3202534),
        (6.5278962, 45.3204607),
        (6.527872, 45.3206),
    ])
}

// https://www.openstreetmap.org/way/200749479
fn area1_0() -> Polygon {
    polygon(&[
        (6.5275881, 45.3134466),
        (6.5278472, 45.3133288),
        (6.5281821, 45.3131695),
        (6.5284257, 45.3131053),
        (6.5286333, 45.3128335),
        (6.5287803, 45.3127847),
        (6.528913, 45.3128392),
        (6.5289579, 45.312911),
        (6.5288926, 45.3130503),
        (6.528425, 45.3135916),
        (6.528162, 45.3140731),
        (6.5281048, 45.3140166),
        (6.5283653, 45.3135486),
        (6.5283781, 45.3135256),
        (6.5286047, 45.3132441),
        (6.5287107, 45.3131179),
        (6.5287987, 45.313013),
        (6.5288477, 45.3129139),
        (6.5288109, 45.3128737),
        (6.5287354, 45.3128751),
        (6.5287193, 45.3128937),
        (6.5285006, 45.3131465),
        (6.5281821, 45.3132628),
        (6.5276549, 45.313516),
        (6.5276364, 45.3134928),
        (6.5275881, 45.3134466),
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

fn segments_enter_leave() -> Vec<TrackSegment> {
    vec![segment(&[
        (6.5233781, 45.3113157),
        (6.5235761, 45.311424),
        (6.5237633, 45.3115298),
        (6.5239088, 45.3116112),
        (6.5240496, 45.3117135),
        (6.5243262, 45.3117365),
        (6.5244469, 45.3118339),
        (6.5246273, 45.3119132),
        (6.5247572, 45.3119803),
        (6.5249056, 45.3120383),
        (6.5250872, 45.312071),
        (6.5252607, 45.3120567),
        (6.5254726, 45.3121102),
        (6.5256625, 45.3121874),
        (6.5258554, 45.3122888),
        (6.5261287, 45.3123904),
        (6.5263775, 45.3125148),
        (6.5264648, 45.3125818),
        (6.5265656, 45.3126546),
        (6.5267541, 45.3128186),
        (6.5268557, 45.3129231),
        (6.5269511, 45.3130399),
        (6.5270301, 45.3131337),
        (6.5271117, 45.3131904),
        (6.5272457, 45.3132652),
        (6.5273506, 45.3133451),
        (6.5275662, 45.3134669),
        (6.5277001, 45.3134394),
        (6.5279491, 45.3133244),
        (6.5284426, 45.3131153),
        (6.5287251, 45.3128361),
        (6.5288191, 45.3130131),
        (6.5285631, 45.3133886),
        (6.528277, 45.3137732),
        (6.5282089, 45.3139223),
        (6.5280927, 45.3141359),
        (6.5281566, 45.3142624),
        (6.5282446, 45.3144319),
        (6.5283465, 45.3145908),
        (6.5284681, 45.3147511),
        (6.5285968, 45.3148795),
        (6.528711, 45.3150084),
        (6.5288402, 45.3151749),
        (6.528908, 45.3153249),
        (6.5290303, 45.3154487),
    ])]
}

#[test]
#[named]
fn enter_and_leave_piste_with_area() {
    let ski_area =
        ski_area(function_name!(), &[piste("1", vec![], vec![area0()])]);
    let segments = segments_enter_leave();
    let expected = vec![
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 5),
        ),
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 27),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 35),
        ),
    ];
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
fn enter_and_leave_piste_with_line() {
    let ski_area =
        ski_area(function_name!(), &[piste("1", vec![line0()], vec![])]);
    let segments = segments_enter_leave();
    let expected = vec![
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 5),
        ),
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 27),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 35),
        ),
    ];
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
fn enter_and_leave_another_piste_with_area() {
    let ski_area = ski_area(
        function_name!(),
        &[
            piste("1", vec![], vec![area0()]),
            piste("2", vec![], vec![area1_0()]),
        ],
    );
    let segments = segments_enter_leave();
    let expected = vec![
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 5),
        ),
        (
            Moving {
                piste_id: "2".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 27),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 35),
        ),
    ];
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
fn enter_and_leave_another_piste_with_line() {
    let ski_area = ski_area(
        function_name!(),
        &[
            piste("1", vec![line0()], vec![]),
            piste("2", vec![line1()], vec![]),
        ],
    );
    let segments = segments_enter_leave();
    let expected = vec![
        (
            Moving {
                piste_id: String::new(),
                move_type: MoveType::Ski,
            },
            (0, 0),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 5),
        ),
        (
            Moving {
                piste_id: "2".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 27),
        ),
        (
            Moving {
                piste_id: "1".to_string(),
                move_type: MoveType::Ski,
            },
            (0, 35),
        ),
    ];
    run_test(
        function_name!(),
        ski_area,
        segments,
        vec![(MoveType::Ski, (0, 0))],
        expected,
    );
}
