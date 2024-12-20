use geo::{BoundingRect, CoordFloat, CoordNum, Rect};
use num_traits::cast::FromPrimitive;
use serde::{Deserialize, Serialize};

use super::rect::expand_rect;
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
        let bounding_rect = item.bounding_rect().into().ok_or_else(|| {
            Error::new_s(
                ErrorType::LogicError,
                "cannot calculate bounding rect",
            )
        })?;
        Ok(BoundedGeometry {
            item,
            bounding_rect,
        })
    }

    pub fn expanded_rect(&self, amount: C) -> Rect<C>
    where
        C: CoordFloat + FromPrimitive,
    {
        let mut rect = self.bounding_rect;
        expand_rect(&mut rect, amount);
        rect
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
