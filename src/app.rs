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
    Selected { rect: Rect },
    PendingCapture { rect_px: RectPx, hidden_at: Instant },
}

pub struct App {
    mode: Mode,
    screenshot: Option<RgbaImage>,
    texture: Option<egui::TextureHandle>,
    image_loaders_installed: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: Mode::Idle,
            screenshot: None,
            texture: None,
            image_loaders_installed: false,
        }
    }
}

fn image_to_texture(ctx: &egui::Context, img: &RgbaImage) -> egui::TextureHandle {
    let size = [img.width() as usize, img.height() as usize];
    let pixels = img.clone().into_raw();
    let color = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
    ctx.load_texture("shot", color, Default::default())
}

/// min/max 分开换算像素，再相减，避免 round(width*ppp) 变大
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

/// 暗幕“挖洞”：只画选区外 4 块暗幕，选区内=桌面原亮度
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
    const CAPTURE_DELAY_MS: u64 = 180;

    fn enter_overlay(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Decorations(false));

        ctx.send_viewport_cmd(ViewportCommand::Maximized(true));
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        ctx.request_repaint();
    }

    fn exit_overlay(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(false));
        ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));

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

    fn overlay_selecting_ui(&mut self, ctx: &egui::Context) {
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

                let (mut start, mut end) = match self.mode {
                    Mode::Selecting { start, end } => (start, end),
                    _ => return,
                };

                if pressed && let Some(p) = pos {
                    start = p;
                    end = p;
                }
                if down && let Some(p) = pos {
                    end = p;
                    ctx.request_repaint();
                }

                if released && let Some(p) = pos {
                    end = p;
                }

                let min = Pos2::new(start.x.min(end.x), start.y.min(end.y));
                let max = Pos2::new(start.x.max(end.x), start.y.max(end.y));
                let sel = Rect::from_min_max(min, max).intersect(full);

                let painter = ui.painter_at(full);
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
                    "Drag to select. Release to continue. Esc to cancel.",
                    egui::FontId::proportional(16.0),
                    Color32::WHITE,
                );

                // 最后写回状态
                if released {
                    if sel.width() < 2.0 || sel.height() < 2.0 {
                        self.mode = Mode::Idle;
                        self.exit_overlay(ctx);
                    } else {
                        self.mode = Mode::Selected { rect: sel };
                        ctx.request_repaint();
                    }
                } else {
                    self.mode = Mode::Selecting { start, end };
                }
            });
    }

    fn overlay_selected_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let full = ui.max_rect();

                let rect = match self.mode {
                    Mode::Selected { rect } => rect,
                    _ => return,
                };

                // Esc = 取消
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.mode = Mode::Idle;
                    self.exit_overlay(ctx);
                    return;
                }

                // 背景暗幕 + 边框
                let painter = ui.painter_at(full);
                paint_dim_with_hole(&painter, full, rect, Self::DIM_ALPHA);
                painter.rect_stroke(
                    rect,
                    0.0,
                    Stroke::new(2.0, Color32::WHITE),
                    StrokeKind::Inside,
                );

                // 工具栏：默认放 = 选区下方居中；如果靠近底部，就放到选区上方
                let btn_size = Vec2::new(32.0, 32.0);
                let spacing = 8.0;

                let toolbar_width = btn_size.x * 3.0 + spacing * 2.0 + 16.0;
                let toolbar_height = btn_size.y + 12.0;

                let mut toolbar_center =
                    Pos2::new(rect.center().x, rect.max.y + 8.0 + toolbar_height / 2.0);
                if toolbar_center.y + toolbar_height / 2.0 > full.max.y - 8.0 {
                    // 放上面
                    toolbar_center.y = rect.min.y - 8.0 - toolbar_height / 2.0;
                }

                // clamp 到屏幕内（避免瞬移/抖动）
                toolbar_center.x = toolbar_center.x.clamp(
                    full.min.x + toolbar_width / 2.0 + 8.0,
                    full.max.x - toolbar_width / 2.0 - 8.0,
                );
                toolbar_center.y = toolbar_center.y.clamp(
                    full.min.y + toolbar_height / 2.0 + 8.0,
                    full.max.y - toolbar_height / 2.0 - 8.0,
                );

                let toolbar_rect = Rect::from_center_size(
                    toolbar_center,
                    Vec2::new(toolbar_width, toolbar_height),
                );
                painter.rect_filled(toolbar_rect, 6.0, Color32::from_rgb(240, 240, 240));

                let btn_center_y = toolbar_rect.center().y;
                let cx = toolbar_rect.center().x;

                // cancel
                let cancel_rect = Rect::from_center_size(
                    Pos2::new(cx - btn_size.x - spacing, btn_center_y),
                    btn_size,
                );
                let cancel_img =
                    egui::Image::new(egui::include_image!("../assets/icons/close.png"))
                        .fit_to_exact_size(btn_size);

                if ui
                    .put(cancel_rect, cancel_img.sense(Sense::click()))
                    .clicked()
                {
                    self.mode = Mode::Idle;
                    self.exit_overlay(ctx);
                    return;
                }

                // confirm
                let confirm_rect = Rect::from_center_size(Pos2::new(cx, btn_center_y), btn_size);
                let check_img = egui::Image::new(egui::include_image!("../assets/icons/check.png"))
                    .fit_to_exact_size(btn_size);

                if ui
                    .put(confirm_rect, check_img.sense(Sense::click()))
                    .clicked()
                {
                    let rect_px = points_rect_to_px(ctx, rect);

                    self.mode = Mode::PendingCapture {
                        rect_px,
                        hidden_at: Instant::now(),
                    };
                    ctx.request_repaint();
                    return;
                }

                // arrow（占位）
                let arrow_rect = Rect::from_center_size(
                    Pos2::new(cx + btn_size.x + spacing, btn_center_y),
                    btn_size,
                );
                let arrow_img = egui::Image::new(egui::include_image!("../assets/icons/arrow.png"))
                    .fit_to_exact_size(btn_size);
                ui.put(arrow_rect, arrow_img.sense(Sense::click()));
            });
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // overlay “洞”里要跟桌面一样亮，这里必须全透明
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // loaders 只装一次
        if !self.image_loaders_installed {
            egui_extras::install_image_loaders(ctx);
            self.image_loaders_installed = true;
        }

        // PendingCapture：这一段期间不画任何东西（全透明窗口），等合成器刷新后截图
        if let Mode::PendingCapture { rect_px, hidden_at } = self.mode {
            // 驱动帧循环
            ctx.request_repaint_after(Duration::from_millis(16));

            if hidden_at.elapsed() < Duration::from_millis(Self::CAPTURE_DELAY_MS) {
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
            Mode::Selecting { .. } => self.overlay_selecting_ui(ctx),
            Mode::Selected { .. } => self.overlay_selected_ui(ctx),
            Mode::PendingCapture { .. } => {}
        }
    }
}
