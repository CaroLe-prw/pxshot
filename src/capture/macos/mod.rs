mod cg;
use crate::capture::{RectPx, RgbaImage};
use anyhow::Result;

pub fn capture_region(rect: RectPx) -> Result<RgbaImage> {
    cg::capture_region_macos(rect)
}
