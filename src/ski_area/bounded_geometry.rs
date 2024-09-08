use geo::{
    BoundingRect, CoordFloat, CoordNum, HaversineBearing, HaversineDestination,
    HaversineDistance, Point, Rect,
};
use num_traits::cast::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorType, Result};

#[derive(Serialize, Deserialize, Debug)]
pub struct BoundedGeometry<T, C = f64>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    pub item: T,
    pub bounding_rect: Rect<C>,
}

impl<T, C> BoundedGeometry<T, C>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    pub fn new(item: T) -> Result<Self> {
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
        let bearing = min_p.haversine_bearing(max_p);
        let distance = min_p.haversine_distance(&max_p);
        self.bounding_rect = Rect::new(
            min_p.haversine_destination(bearing, -amount).into(),
            max_p.haversine_destination(bearing, distance + amount),
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
