use anyhow::Result;

use crate::capture::{RectPx, RgbaImage};

mod x11;

pub fn capture_region(rect: RectPx) -> Result<RgbaImage> {
    x11::capture_region_x11(rect)
}
