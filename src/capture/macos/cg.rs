use anyhow::Result;
use image::{ImageBuffer, Rgba};

use crate::capture::{RectPx, RgbaImage};
use core_graphics::display::CGDisplay;
use core_graphics::geometry::{CGPoint, CGRect, CGSize};

pub fn capture_region_macos(r: RectPx) -> Result<RgbaImage> {
    let rect = CGRect::new(
        &CGPoint::new(r.x as f64, r.y as f64),
        &CGSize::new(r.w as f64, r.h as f64),
    );

    let cgimg = CGDisplay::main()
        .create_image_for_rect(rect)
        .ok_or_else(|| anyhow::anyhow!("create_image_for_rect failed"))?;

    let width = cgimg.width() as u32;
    let height = cgimg.height() as u32;
    let bytes_per_row = cgimg.bytes_per_row() as usize;

    // core-graphics 0.25：CGImageRef::data() -> CFData（拥有底层 buffer）:contentReference[oaicite:9]{index=9}
    let data = cgimg.data();
    let src: &[u8] = data.bytes();

    // 通常是 BGRA / premultiplied alpha
    let mut out = vec![0u8; (width * height * 4) as usize];

    for y in 0..height as usize {
        let row_src = &src[y * bytes_per_row..y * bytes_per_row + width as usize * 4];
        let row_dst = &mut out[y * (width as usize * 4)..(y + 1) * (width as usize * 4)];

        for (s, d) in row_src.chunks_exact(4).zip(row_dst.chunks_exact_mut(4)) {
            let b = s[0];
            let g = s[1];
            let rr = s[2];
            let a = s[3];
            d[0] = rr;
            d[1] = g;
            d[2] = b;
            d[3] = a;
        }
    }

    Ok(ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, out)
        .ok_or_else(|| anyhow::anyhow!("bad buffer size"))?)
}
