//! Splitting a screen area into four equal quadrants, one per watched bot.

use bevy::prelude::*;
use std::fmt;
use std::str::FromStr;

/// One quarter of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Quadrant {
    /// Reading order, so the tiers fill the screen novice to expert.
    pub const ALL: [Self; 4] = [
        Self::TopLeft,
        Self::TopRight,
        Self::BottomLeft,
        Self::BottomRight,
    ];

    /// The part of `area` this quadrant covers. Odd sizes give the extra
    /// pixel to the right and bottom quadrants, so the four tile `area`
    /// exactly.
    pub fn rect(self, area: IRect) -> IRect {
        let mid = area.min + area.size() / 2;
        let (x, y) = match self {
            Self::TopLeft => ((area.min.x, mid.x), (area.min.y, mid.y)),
            Self::TopRight => ((mid.x, area.max.x), (area.min.y, mid.y)),
            Self::BottomLeft => ((area.min.x, mid.x), (mid.y, area.max.y)),
            Self::BottomRight => ((mid.x, area.max.x), (mid.y, area.max.y)),
        };
        IRect::new(x.0, y.0, x.1, y.1)
    }

    /// Indexed by discriminant, in the order of the variants.
    const NAMES: [&str; 4] = ["top-left", "top-right", "bottom-left", "bottom-right"];
}

impl fmt::Display for Quadrant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Self::NAMES[*self as usize])
    }
}

impl FromStr for Quadrant {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .zip(Self::NAMES)
            .find(|(_, name)| name.eq_ignore_ascii_case(s))
            .map(|(q, _)| q)
            .ok_or_else(|| {
                format!(
                    "unknown quadrant {s:?}; expected {}",
                    Self::NAMES.join(", ")
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadrants_tile_the_area_exactly() {
        // A 1920x1080 screen minus a 48 px taskbar, with an odd height.
        let area = IRect::new(0, 0, 1920, 1031);
        let rects = Quadrant::ALL.map(|q| q.rect(area));
        let total: i32 = rects.iter().map(|r| r.width() * r.height()).sum();
        assert_eq!(total, area.width() * area.height());
        assert_eq!(rects[0], IRect::new(0, 0, 960, 515));
        assert_eq!(rects[3], IRect::new(960, 515, 1920, 1031));
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                assert!(a.intersect(*b).is_empty(), "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn offset_area_keeps_its_origin() {
        // Taskbar on the left: the work area starts at x = 64.
        let area = IRect::new(64, 0, 1984, 1080);
        assert_eq!(Quadrant::TopLeft.rect(area).min, IVec2::new(64, 0));
        assert_eq!(Quadrant::BottomRight.rect(area).max, IVec2::new(1984, 1080));
    }

    #[test]
    fn names_round_trip() {
        for q in Quadrant::ALL {
            assert_eq!(q.to_string().parse::<Quadrant>(), Ok(q));
        }
        assert!("middle".parse::<Quadrant>().is_err());
    }
}
