use eframe::egui::{self, Rect, StrokeKind, ViewportCommand};
use egui::{Color32, Stroke};
use std::time::Instant;

use super::paint_dim_with_hole;
use crate::App;
use crate::mode::Mode;
use crate::overlay::toolbar::{Toolbar, ToolbarAction};
use crate::overlay::{DIM_ALPHA, HitZone, draw_size_label, points_rect_to_px};

impl App {
    pub fn overlay_selected_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let full = ui.max_rect();

                let (mut rect, dragging) = match self.mode {
                    Mode::Selected { rect, dragging } => (rect, dragging),
                    _ => return,
                };

                // Esc = 取消
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.cancel_overlay(ctx);
                    return;
                }

                // 获取鼠标状态
                let (pos, pressed, down, released, delta) = ctx.input(|i| {
                    (
                        i.pointer.interact_pos(),
                        i.pointer.primary_pressed(),
                        i.pointer.primary_down(),
                        i.pointer.primary_released(),
                        i.pointer.delta(),
                    )
                });

                // 检测当前区域
                let current_zone = pos
                    .map(|p| HitZone::detect(p, rect))
                    .unwrap_or(HitZone::None);

                // 设置光标
                let cursor_zone = dragging.unwrap_or(current_zone);
                ctx.set_cursor_icon(cursor_zone.cursor());

                // 处理拖拽
                let mut new_dragging = dragging;

                if pressed && current_zone != HitZone::None {
                    // 开始拖拽
                    new_dragging = Some(current_zone);
                }

                if down && let Some(zone) = new_dragging {
                    // 拖拽中，更新选区
                    rect = zone.apply_drag(rect, delta, full);
                    ctx.request_repaint();
                }

                if released {
                    // 结束拖拽
                    new_dragging = None;
                }

                // 更新状态
                self.mode = Mode::Selected {
                    rect,
                    dragging: new_dragging,
                };

                // 绘制暗幕和边框
                let painter = ui.painter_at(full);
                paint_dim_with_hole(&painter, full, rect, DIM_ALPHA);
                painter.rect_stroke(
                    rect,
                    0.0,
                    Stroke::new(2.0, Color32::WHITE),
                    StrokeKind::Inside,
                );

                // 显示尺寸
                draw_size_label(&painter, ctx, rect);

                // 绘制调整手柄
                draw_resize_handles(&painter, rect);

                // 工具栏（只在不拖拽时响应）
                if new_dragging.is_none() {
                    let toolbar = Toolbar::default();
                    let toolbar_rect = toolbar.calc_rect(rect, full);

                    match toolbar.show(ui, toolbar_rect) {
                        ToolbarAction::Cancel => {
                            self.cancel_overlay(ctx);
                        }
                        ToolbarAction::Confirm => {
                            let rect_px = points_rect_to_px(ctx, rect);
                            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                            self.mode = Mode::PendingCapture {
                                rect_px,
                                hidden_at: Instant::now(),
                            };
                            ctx.request_repaint();
                        }
                        ToolbarAction::Arrow => {}
                        ToolbarAction::None => {}
                    }
                }
            });
    }
}

/// 绘制四角的调整手柄
fn draw_resize_handles(painter: &egui::Painter, rect: Rect) {
    let handle_size = 6.0;
    let color = Color32::WHITE;

    let corners = [
        rect.left_top(),
        rect.right_top(),
        rect.left_bottom(),
        rect.right_bottom(),
    ];

    for corner in corners {
        let handle_rect = Rect::from_center_size(corner, egui::vec2(handle_size, handle_size));
        painter.rect_filled(handle_rect, 1.0, color);
    }
}
