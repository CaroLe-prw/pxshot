// src/app.rs
use eframe::egui::{self, StrokeKind};
use egui::viewport::{ViewportCommand, WindowLevel};
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use std::time::{Duration, Instant};

use crate::capture::{RectPx, RgbaImage, capture_region};

#[derive(Debug, Clone, Copy)]
enum Mode {
    Idle,
    Selecting { start: Pos2, end: Pos2 },
    PendingCapture { rect_px: RectPx, hidden_at: Instant },
}

pub struct App {
    mode: Mode,
    screenshot: Option<RgbaImage>,
    texture: Option<egui::TextureHandle>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: Mode::Idle,
            screenshot: None,
            texture: None,
        }
    }
}

fn image_to_texture(ctx: &egui::Context, img: &RgbaImage) -> egui::TextureHandle {
    let size = [img.width() as usize, img.height() as usize];
    let pixels = img.clone().into_raw();
    let color = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
    ctx.load_texture("shot", color, Default::default())
}

fn points_rect_to_px(ctx: &egui::Context, r: Rect) -> RectPx {
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

fn paint_dim_with_hole(p: &egui::Painter, full: Rect, hole: Rect, alpha: u8) {
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

impl App {
    const DIM_ALPHA: u8 = 120;

    fn enter_overlay(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(true));
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        ctx.request_repaint();
    }

    fn exit_overlay(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.request_repaint();
    }

    fn idle_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("pxshot");

            if ui.button("Region screenshot").clicked() {
                self.screenshot = None;
                self.texture = None;
                self.enter_overlay(ctx);
                self.mode = Mode::Selecting {
                    start: Pos2::new(0.0, 0.0),
                    end: Pos2::new(0.0, 0.0),
                };
            }

            if let Some(img) = &self.screenshot
                && self.texture.is_none()
            {
                self.texture = Some(image_to_texture(ctx, img));
            }

            if let Some(tex) = &self.texture {
                ui.separator();
                ui.label(format!("Captured: {}x{}", tex.size()[0], tex.size()[1]));
                ui.image((tex.id(), tex.size_vec2()));

                if ui.button("Save screenshot.png").clicked()
                    && let Some(img) = &self.screenshot
                {
                    let _ = img.save("screenshot.png");
                }
            }
        });
    }

    fn overlay_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let full = ui.max_rect();
                let resp = ui.allocate_rect(full, Sense::click_and_drag());

                let (pos, pressed, down, released, esc) = ctx.input(|i| {
                    (
                        i.pointer.interact_pos(),
                        i.pointer.primary_pressed(),
                        i.pointer.primary_down(),
                        i.pointer.primary_released(),
                        i.key_pressed(egui::Key::Escape),
                    )
                });

                if esc {
                    self.mode = Mode::Idle;
                    self.exit_overlay(ctx);
                    return;
                }

                if resp.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
                }

                if let Mode::Selecting { start, end } = &mut self.mode {
                    if pressed && let Some(p) = pos {
                        *start = p;
                        *end = p;
                    }

                    if down && let Some(p) = pos {
                        *end = p;
                        ctx.request_repaint();
                    }

                    if released {
                        let min = Pos2::new(start.x.min(end.x), start.y.min(end.y));
                        let max = Pos2::new(start.x.max(end.x), start.y.max(end.y));
                        let sel = Rect::from_min_max(min, max).intersect(full);

                        if sel.width() < 2.0 || sel.height() < 2.0 {
                            self.mode = Mode::Idle;
                            self.exit_overlay(ctx);
                            return;
                        }

                        let rect_px = points_rect_to_px(ctx, sel);

                        // 先隐藏 overlay，避免把暗幕/边框截进去
                        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                        ctx.request_repaint_after(Duration::from_millis(160));

                        self.mode = Mode::PendingCapture {
                            rect_px,
                            hidden_at: Instant::now(),
                        };
                        return;
                    }

                    // 绘制：暗幕挖洞 + 边框
                    let painter = ui.painter_at(full);

                    let min = Pos2::new(start.x.min(end.x), start.y.min(end.y));
                    let max = Pos2::new(start.x.max(end.x), start.y.max(end.y));
                    let sel = Rect::from_min_max(min, max).intersect(full);

                    if sel.width() > 1.0 && sel.height() > 1.0 {
                        paint_dim_with_hole(&painter, full, sel, Self::DIM_ALPHA);

                        painter.rect_stroke(
                            sel,
                            0.0,
                            Stroke::new(2.0, Color32::WHITE),
                            StrokeKind::Inside,
                        );
                    } else {
                        painter.rect_filled(full, 0.0, Color32::from_black_alpha(Self::DIM_ALPHA));
                    }

                    painter.text(
                        full.center_top() + Vec2::new(0.0, 12.0),
                        egui::Align2::CENTER_TOP,
                        "Drag to select. Release to capture. Esc to cancel.",
                        egui::FontId::proportional(16.0),
                        Color32::WHITE,
                    );
                }
            });
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Mode::PendingCapture { rect_px, hidden_at } = self.mode {
            if hidden_at.elapsed() < Duration::from_millis(160) {
                return;
            }

            match capture_region(rect_px) {
                Ok(img) => {
                    self.screenshot = Some(img);
                    self.texture = None;
                }
                Err(e) => eprintln!("capture failed: {e:?}"),
            }

            self.mode = Mode::Idle;
            self.exit_overlay(ctx);
            return;
        }

        match self.mode {
            Mode::Idle => self.idle_ui(ctx),
            Mode::Selecting { .. } => self.overlay_ui(ctx),
            Mode::PendingCapture { .. } => {}
        }
    }
}
