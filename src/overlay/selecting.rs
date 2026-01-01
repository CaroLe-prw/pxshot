use eframe::egui::{self, StrokeKind};
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};

use super::{DIM_ALPHA, paint_dim_with_hole};
use crate::App;
use crate::mode::Mode;
use crate::overlay::draw_size_label;

impl App {
    pub fn overlay_selecting_ui(&mut self, ctx: &egui::Context) {
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
                    self.cancel_overlay(ctx);
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
                    paint_dim_with_hole(&painter, full, sel, DIM_ALPHA);
                    painter.rect_stroke(
                        sel,
                        0.0,
                        Stroke::new(2.0, Color32::WHITE),
                        StrokeKind::Inside,
                    );

                    draw_size_label(&painter, ctx, sel);
                } else {
                    painter.rect_filled(full, 0.0, Color32::from_black_alpha(DIM_ALPHA));
                }

                painter.text(
                    full.center_top() + Vec2::new(0.0, 12.0),
                    egui::Align2::CENTER_TOP,
                    "Drag to select. Release to continue. Esc to cancel.",
                    egui::FontId::proportional(16.0),
                    Color32::WHITE,
                );

                if released {
                    if sel.width() < 2.0 || sel.height() < 2.0 {
                        self.mode = Mode::Idle;
                        self.exit_overlay(ctx);
                    } else {
                        self.mode = Mode::Selected {
                            rect: sel,
                            dragging: None,
                        };
                        ctx.request_repaint();
                    }
                } else {
                    self.mode = Mode::Selecting { start, end };
                }
            });
    }
}
