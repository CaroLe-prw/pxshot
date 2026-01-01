use anyhow::Result;
use arboard::{Clipboard, ImageData};

use crate::capture::RgbaImage;

pub fn copy_image(img: &RgbaImage) -> Result<()> {
    let mut clipboard = Clipboard::new()?;

    let img_data = ImageData {
        width: img.width() as usize,
        height: img.height() as usize,
        bytes: img.as_raw().into(),
    };
    clipboard.set_image(img_data)?;
    Ok(())
}
