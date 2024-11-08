use geo::{coord, CoordFloat, Destination, Haversine, Point, Rect};
use num_traits::cast::FromPrimitive;

pub fn union_rects<C>(r1: Rect<C>, r2: Rect<C>) -> Rect<C>
where
    C: CoordFloat,
{
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

pub fn expand_rect<C>(rect: &mut Rect<C>, amount: C)
where
    C: CoordFloat + FromPrimitive,
{
    let min_p = Point::from(rect.min());
    let max_p = Point::from(rect.max());
    let min_x = Haversine::destination(min_p, C::from(-90.0).unwrap(), amount);
    let min_y = Haversine::destination(min_p, C::from(180.0).unwrap(), amount);
    let max_x = Haversine::destination(max_p, C::from(90.0).unwrap(), amount);
    let max_y = Haversine::destination(max_p, C::from(0.0).unwrap(), amount);

    rect.set_min(coord! { x: min_x.x(), y: min_y.y() });
    rect.set_max(coord! { x: max_x.x(), y: max_y.y() });
}

pub fn union_rects_if<C>(
    r1: Option<Rect<C>>,
    r2: Option<Rect<C>>,
) -> Option<Rect<C>>
where
    C: CoordFloat,
{
    match (r1, r2) {
        (None, None) => None,
        (Some(r), None) => Some(r),
        (None, Some(r)) => Some(r),
        (Some(r1), Some(r2)) => Some(union_rects(r1, r2)),
    }
}

pub fn union_rects_all<It, C>(it: It) -> Option<Rect<C>>
where
    It: Iterator<Item = Rect<C>>,
    C: CoordFloat,
{
    it.reduce(union_rects)
}

//pub fn union_bounding_rects_all<'a, It, C, T>(it: It) -> Option<Rect<C>>
//where
//    It: Iterator<Item = &'a T>,
//    T: BoundingRect<C> + 'a,
//    C: CoordFloat,
//{
//    it.map(|x| x.bounding_rect().into())
//        .reduce(union_rects_if)?
//}
