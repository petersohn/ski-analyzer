use geo::{BoundingRect, Intersects, Rect};
use serde::{Deserialize, Serialize};

#[cfg(feature = "specta")]
use specta::Type;

use super::rect::expand_rect;
use crate::error::{Error, ErrorType, Result};

#[cfg_attr(feature = "specta", derive(Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "BoundedGeometry")]
pub struct BoundedGeometry<T> {
    pub item: T,
    #[cfg_attr(feature = "specta", specta(type = crate::typescript_gen::geo::RectDef))]
    pub bounding_rect: Rect,
}

impl<T> BoundedGeometry<T>
where
    T: BoundingRect<f64>,
{
    pub fn new(item: T) -> Result<Self>
    where
        T: BoundingRect<f64>,
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

    pub fn expanded_rect(&self, amount: f64) -> Rect {
        let mut rect = self.bounding_rect;
        expand_rect(&mut rect, amount);
        rect
    }
}

impl<T> BoundingRect<f64> for BoundedGeometry<T>
where
    T: BoundingRect<f64>,
{
    type Output = Rect;

    fn bounding_rect(&self) -> Self::Output {
        self.bounding_rect.bounding_rect()
    }
}

impl<T> PartialEq for BoundedGeometry<T>
where
    T: BoundingRect<f64> + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

impl<T, U> Intersects<BoundedGeometry<U>> for BoundedGeometry<T>
where
    T: BoundingRect<f64> + Intersects<U>,
    U: BoundingRect<f64>,
{
    fn intersects(&self, rhs: &BoundedGeometry<U>) -> bool {
        self.bounding_rect.intersects(&rhs.bounding_rect)
            && self.item.intersects(&rhs.item)
    }
}
