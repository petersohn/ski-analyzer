use super::{Activity, ActivityType, Segments};
use crate::error::Result;
use crate::utils::cancel::CancellationToken;

fn find_pistes(
    cancel: &CancellationToken,
    segments: Segments,
) -> Result<Segments> {
    let result = Vec::new();

    Ok(Segments::new(result))
}
