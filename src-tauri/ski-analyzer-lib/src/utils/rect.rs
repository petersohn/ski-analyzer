use geo::{coord, Rect};

pub fn union_rects(r1: Rect, r2: Rect) -> Rect {
    Rect::new(
        coord! {
            x: r1.min().x.min(r2.min().x),
            y: r1.min().y.min(r2.min().y),
        },
        coord! {
            x: r1.max().x.max(r2.max().x),
            y: r1.max().y.max(r2.max().y),
        },
    )
}

pub fn union_rects_if(r1: Option<Rect>, r2: Option<Rect>) -> Option<Rect> {
    match (r1, r2) {
        (None, None) => None,
        (Some(r), None) => Some(r),
        (None, Some(r)) => Some(r),
        (Some(r1), Some(r2)) => Some(union_rects(r1, r2)),
    }
}
