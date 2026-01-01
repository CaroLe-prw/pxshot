use anyhow::Result;
use image::{ImageBuffer, Rgba};

pub type RgbaImage = ImageBuffer<Rgba<u8>, Vec<u8>>;

#[derive(Clone, Copy, Debug)]
pub struct RectPx {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub fn capture_region(rect: RectPx) -> Result<RgbaImage> {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return crate::capture::linux::capture_region(rect);
    }

    #[cfg(windows)]
    {
        return crate::capture::windows::capture_region(rect);
    }

    #[cfg(target_os = "macos")]
    {
        return crate::capture::macos::capture_region(rect);
    }

    #[allow(unreachable_code)]
    Err(anyhow::anyhow!("unsupported platform"))
}

#[cfg(all(unix, not(target_os = "macos")))]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(windows)]
pub mod windows;
