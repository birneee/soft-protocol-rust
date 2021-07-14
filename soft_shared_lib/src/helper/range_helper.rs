use std::ops::Range;
use crate::helper::range_helper::RangeCompare::{LOWER, HIGHER, CONTAINED};

pub enum RangeCompare {
    /// value is lower than range
    LOWER,
    /// value is in range
    CONTAINED,
    /// value is higher than range
    HIGHER,
}

/// check if value is lower, contained or higher than the range
pub fn compare_range<T>(range: &Range<T>, value: T) -> RangeCompare
    where T: PartialOrd<T>
{
    if value < range.start {
        LOWER
    } else if value >= range.end {
        HIGHER
    } else {
        CONTAINED
    }
}