use geo::{
    coord, BoundingRect, CoordFloat, CoordNum, Destination, Haversine, Point,
    Rect,
};
use num_traits::cast::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorType, Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoundedGeometry<T, C = f64>
where
    C: CoordNum,
{
    pub item: T,
    pub bounding_rect: Rect<C>,
}

impl<T, C> BoundedGeometry<T, C>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    pub fn new(item: T) -> Result<Self>
    where
        T: BoundingRect<C>,
    {
        let bounding_rect = item.bounding_rect().into().ok_or(Error::new_s(
            ErrorType::LogicError,
            "cannot calculate bounding rect",
        ))?;
        Ok(BoundedGeometry {
            item,
            bounding_rect,
        })
    }

    pub fn expand(&mut self, amount: C)
    where
        C: CoordFloat + FromPrimitive,
    {
        let min_p = Point::from(self.bounding_rect.min());
        let max_p = Point::from(self.bounding_rect.max());
        let min_x =
            Haversine::destination(min_p, C::from(-90.0).unwrap(), amount);
        let min_y =
            Haversine::destination(min_p, C::from(180.0).unwrap(), amount);
        let max_x =
            Haversine::destination(max_p, C::from(90.0).unwrap(), amount);
        let max_y =
            Haversine::destination(max_p, C::from(0.0).unwrap(), amount);
        self.bounding_rect = Rect::new(
            coord! { x: min_x.x(), y: min_y.y() },
            coord! { x: max_x.x(), y: max_y.y() },
        );
    }
}

impl<T, C> BoundingRect<C> for BoundedGeometry<T, C>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    type Output = Rect<C>;

    fn bounding_rect(&self) -> Self::Output {
        self.bounding_rect.bounding_rect()
    }
}

impl<T, C> PartialEq for BoundedGeometry<T, C>
where
    C: CoordNum,
    T: BoundingRect<C> + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}
