//! Hit detection on the ground plane. Circles in integer millimeters.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Circle {
    pub x: i64,
    pub y: i64,
    pub radius: i64,
}

impl Circle {
    /// Touching circles count as overlapping. Negative radii count as zero.
    pub fn overlaps(&self, other: &Circle) -> bool {
        let dx = u128::from(self.x.abs_diff(other.x));
        let dy = u128::from(self.y.abs_diff(other.y));
        let reach = u128::from(self.radius.max(0).unsigned_abs())
            + u128::from(other.radius.max(0).unsigned_abs());
        (dx * dx).saturating_add(dy * dy) <= reach * reach
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circle(x: i64, y: i64, radius: i64) -> Circle {
        Circle { x, y, radius }
    }

    #[test]
    fn separated_circles_do_not_overlap() {
        assert!(!circle(0, 0, 10).overlaps(&circle(30, 0, 10)));
    }

    #[test]
    fn touching_circles_overlap() {
        assert!(circle(0, 0, 10).overlaps(&circle(20, 0, 10)));
    }

    #[test]
    fn overlap_is_symmetric() {
        let a = circle(0, 0, 10);
        let b = circle(12, 9, 6);
        assert_eq!(a.overlaps(&b), b.overlaps(&a));
        assert!(a.overlaps(&b));
    }

    #[test]
    fn huge_coordinates_do_not_overflow() {
        let far = circle(i64::MAX, i64::MAX, 1);
        assert!(!circle(i64::MIN, i64::MIN, 1).overlaps(&far));
    }
}
