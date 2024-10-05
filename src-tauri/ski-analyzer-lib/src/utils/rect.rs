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
