use anyhow::Result;
use image::{ImageBuffer, Rgba};

use crate::capture::{RectPx, RgbaImage};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;

pub fn capture_region_windows(r: RectPx) -> Result<RgbaImage> {
    unsafe {
        let hwnd: HWND = GetDesktopWindow();
        let hdc_screen = GetDC(hwnd);
        if hdc_screen.0 == 0 {
            anyhow::bail!("GetDC failed");
        }

        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbmp = CreateCompatibleBitmap(hdc_screen, r.w as i32, r.h as i32);

        let old = SelectObject(hdc_mem, hbmp);

        let ok = BitBlt(
            hdc_mem, 0, 0, r.w as i32, r.h as i32, hdc_screen, r.x as i32, r.y as i32, SRCCOPY,
        );
        if !ok.as_bool() {
            anyhow::bail!("BitBlt failed");
        }

        let mut bmi = BITMAPINFO::default();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = r.w as i32;
        bmi.bmiHeader.biHeight = -(r.h as i32); // top-down
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB.0 as u32;

        let mut buf = vec![0u8; (r.w * r.h * 4) as usize];
        let got = GetDIBits(
            hdc_mem,
            hbmp,
            0,
            r.h,
            Some(buf.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );
        if got == 0 {
            anyhow::bail!("GetDIBits failed");
        }

        // BGRA -> RGBA
        for px in buf.chunks_exact_mut(4) {
            let b = px[0];
            let g = px[1];
            let rr = px[2];
            let a = px[3];
            px[0] = rr;
            px[1] = g;
            px[2] = b;
            px[3] = a;
        }

        SelectObject(hdc_mem, old);
        DeleteObject(hbmp);
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_screen);

        Ok(ImageBuffer::<Rgba<u8>, _>::from_raw(r.w, r.h, buf)
            .ok_or_else(|| anyhow::anyhow!("bad buffer size"))?)
    }
}
