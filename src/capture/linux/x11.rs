use anyhow::Result;
use image::{ImageBuffer, Rgba};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, ImageFormat};

use crate::capture::{RectPx, RgbaImage};

pub fn capture_region_x11(r: RectPx) -> Result<RgbaImage> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let reply = conn
        .get_image(
            ImageFormat::Z_PIXMAP,
            root,
            r.x as i16,
            r.y as i16,
            r.w as u16,
            r.h as u16,
            u32::MAX,
        )?
        .reply()?;

    // 常见桌面：每像素 4 字节 B,G,R,X
    let mut out = Vec::with_capacity((r.w * r.h * 4) as usize);
    for px in reply.data.chunks_exact(4) {
        let b = px[0];
        let g = px[1];
        let rr = px[2];
        out.extend_from_slice(&[rr, g, b, 255]);
    }

    ImageBuffer::<Rgba<u8>, _>::from_raw(r.w, r.h, out)
        .ok_or_else(|| anyhow::anyhow!("bad buffer size"))
}
