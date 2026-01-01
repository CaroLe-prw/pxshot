mod gdi;
use crate::capture::{RectPx, RgbaImage};
use anyhow::Result;

pub fn capture_region(rect: RectPx) -> Result<RgbaImage> {
    gdi::capture_region_windows(rect)
}
