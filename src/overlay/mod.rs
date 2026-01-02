mod resize;
mod selected;
mod selecting;
mod toolbar;

use crate::capture::RectPx;
use eframe::egui::{self, Color32, Pos2, Rect};
pub use resize::HitZone;
pub const DIM_ALPHA: u8 = 120;

/// 使用单一 Mesh 绘制带洞的暗幕，避免多块分离导致的不同步消失
pub fn paint_dim_with_hole(p: &egui::Painter, full: Rect, hole: Rect, alpha: u8) {
    use egui::epaint::{Mesh, Vertex};

    let hole = hole.intersect(full);
    let dim = Color32::from_black_alpha(alpha);

    if hole.width() <= 0.0 || hole.height() <= 0.0 {
        p.rect_filled(full, 0.0, dim);
        return;
    }

    // 构建单一带洞的 Mesh：外框 4 顶点 + 内框 4 顶点
    // 使用三角形带连接外框和内框，形成一个整体的"相框"形状
    let mut mesh = Mesh::default();
    let uv = egui::epaint::WHITE_UV;

    // 外框顶点 (0-3)
    mesh.vertices.push(Vertex {
        pos: full.left_top(),
        uv,
        color: dim,
    }); // 0
    mesh.vertices.push(Vertex {
        pos: full.right_top(),
        uv,
        color: dim,
    }); // 1
    mesh.vertices.push(Vertex {
        pos: full.right_bottom(),
        uv,
        color: dim,
    }); // 2
    mesh.vertices.push(Vertex {
        pos: full.left_bottom(),
        uv,
        color: dim,
    }); // 3

    // 内框顶点 (4-7)
    mesh.vertices.push(Vertex {
        pos: hole.left_top(),
        uv,
        color: dim,
    }); // 4
    mesh.vertices.push(Vertex {
        pos: hole.right_top(),
        uv,
        color: dim,
    }); // 5
    mesh.vertices.push(Vertex {
        pos: hole.right_bottom(),
        uv,
        color: dim,
    }); // 6
    mesh.vertices.push(Vertex {
        pos: hole.left_bottom(),
        uv,
        color: dim,
    }); // 7

    // 用三角形连接外框和内框的对应边，形成 4 个梯形区域
    // 上边区域：0-1-5-4
    mesh.indices.extend_from_slice(&[0, 1, 5, 0, 5, 4]);
    // 右边区域：1-2-6-5
    mesh.indices.extend_from_slice(&[1, 2, 6, 1, 6, 5]);
    // 下边区域：2-3-7-6
    mesh.indices.extend_from_slice(&[2, 3, 7, 2, 7, 6]);
    // 左边区域：3-0-4-7
    mesh.indices.extend_from_slice(&[3, 0, 4, 3, 4, 7]);

    p.add(egui::Shape::mesh(mesh));
}

/// 获取选择框的size
pub fn points_rect_to_px(ctx: &egui::Context, r: Rect) -> RectPx {
    let ppp = ctx.pixels_per_point();

    let x = (r.min.x * ppp).round().max(0.0) as u32;
    let y = (r.min.y * ppp).round().max(0.0) as u32;
    let w = (r.width() * ppp).round().max(1.0) as u32;
    let h = (r.height() * ppp).round().max(1.0) as u32;

    RectPx { x, y, w, h }
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
