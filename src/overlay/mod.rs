mod resize;
mod selected;
mod selecting;
mod toolbar;

use crate::capture::RectPx;
use eframe::egui::{self, Color32, Pos2, Rect};
pub use resize::HitZone;
pub const DIM_ALPHA: u8 = 120;

/// 只画选区外 4 块暗幕，选区内=桌面原亮度
pub fn paint_dim_with_hole(p: &egui::Painter, full: Rect, hole: Rect, alpha: u8) {
    let hole = hole.intersect(full);
    let dim = Color32::from_black_alpha(alpha);

    if hole.width() <= 0.0 || hole.height() <= 0.0 {
        p.rect_filled(full, 0.0, dim);
        return;
    }

    // 上
    p.rect_filled(
        Rect::from_min_max(full.min, Pos2::new(full.max.x, hole.min.y)),
        0.0,
        dim,
    );
    // 下
    p.rect_filled(
        Rect::from_min_max(Pos2::new(full.min.x, hole.max.y), full.max),
        0.0,
        dim,
    );
    // 左
    p.rect_filled(
        Rect::from_min_max(
            Pos2::new(full.min.x, hole.min.y),
            Pos2::new(hole.min.x, hole.max.y),
        ),
        0.0,
        dim,
    );
    // 右
    p.rect_filled(
        Rect::from_min_max(
            Pos2::new(hole.max.x, hole.min.y),
            Pos2::new(full.max.x, hole.max.y),
        ),
        0.0,
        dim,
    );
}

/// 获取选择框的size
pub fn points_rect_to_px(ctx: &egui::Context, r: Rect) -> RectPx {
    let ppp = ctx.pixels_per_point();

    let x1 = (r.min.x * ppp).floor().max(0.0) as i32;
    let y1 = (r.min.y * ppp).floor().max(0.0) as i32;
    let x2 = (r.max.x * ppp).ceil().max(0.0) as i32;
    let y2 = (r.max.y * ppp).ceil().max(0.0) as i32;

    RectPx {
        x: x1.max(0) as u32,
        y: y1.max(0) as u32,
        w: (x2 - x1).max(1) as u32,
        h: (y2 - y1).max(1) as u32,
    }
}

/// 绘制选区尺寸标签
pub fn draw_size_label(painter: &egui::Painter, ctx: &egui::Context, rect: Rect) {
    let rect_px = points_rect_to_px(ctx, rect);
    let size_text = format!("{}×{}", rect_px.w, rect_px.h);

    let text_pos = if rect.min.y < 30.0 {
        Pos2::new(rect.min.x + 4.0, rect.min.y + 18.0)
    } else {
        Pos2::new(rect.min.x, rect.min.y - 6.0)
    };
    let font = egui::FontId::proportional(14.0);
    painter.text(
        text_pos,
        egui::Align2::LEFT_BOTTOM,
        size_text,
        font,
        Color32::WHITE,
    );
}
