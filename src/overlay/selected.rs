use eframe::egui::{self, CursorIcon, Rect, StrokeKind, ViewportCommand};
use egui::{Color32, Stroke};
use std::time::Instant;

use super::paint_dim_with_hole;
use crate::App;
use crate::mode::Mode;
use crate::overlay::toolbar::{Toolbar, ToolbarAction};
use crate::overlay::{DIM_ALPHA, HitZone, draw_size_label, points_rect_to_px};
use crate::tools::arrow::{DrawState, PopupState};

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

                // Esc = 取消绘制或退出
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    if self.arrow_mode_active {
                        // 如果正在绘制，取消绘制
                        if self.arrow_drawer.state == DrawState::Drawing {
                            self.arrow_drawer.cancel();
                        } else {
                            // 退出箭头模式
                            self.arrow_mode_active = false;
                            self.arrow_drawer.state = DrawState::Idle;
                        }
                    } else {
                        self.cancel_overlay(ctx);
                        return;
                    }
                }

                // Delete 键删除选中的箭头
                if ctx.input(|i| {
                    i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
                }) {
                    self.arrow_drawer.delete_selected();
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

                // 绘制暗幕和边框
                let painter = ui.painter_at(full);
                paint_dim_with_hole(&painter, full, rect, DIM_ALPHA);
                painter.rect_stroke(
                    rect,
                    0.0,
                    Stroke::new(2.0, Color32::WHITE),
                    StrokeKind::Inside,
                );

                // 绘制所有已绘制的箭头
                self.arrow_drawer.draw_all(&painter);
                self.arrow_drawer.draw_selection_handles(&painter);

                // 绘制正在绘制的箭头预览
                if let Some(current_pos) = pos {
                    self.arrow_drawer
                        .draw_preview(&painter, current_pos, &self.arrow_panel.config);
                }

                // 工具栏
                let toolbar = Toolbar::default();
                let toolbar_rect = toolbar.calc_rect(rect, full);
                let arrow_panel_rect = self.arrow_panel.calc_panel_rect(toolbar_rect, full);

                // 检查鼠标是否在 UI 区域内
                let in_ui_area = pos.is_some_and(|p| {
                    toolbar_rect.contains(p)
                        || (self.show_arrow_panel && arrow_panel_rect.contains(p))
                });

                // 箭头模式的鼠标处理
                if self.arrow_mode_active && !in_ui_area {
                    if let Some(mouse_pos) = pos {
                        // 只在选区内响应
                        if rect.contains(mouse_pos) {
                            self.handle_arrow_input(ctx, mouse_pos, pressed, down, released);
                        }
                    }
                } else if !self.arrow_mode_active && !in_ui_area {
                    // 非箭头模式：处理选区拖拽
                    let current_zone = pos
                        .map(|p| HitZone::detect(p, rect))
                        .unwrap_or(HitZone::None);

                    let cursor_zone = dragging.unwrap_or(current_zone);
                    ctx.set_cursor_icon(cursor_zone.cursor());

                    let mut new_dragging = dragging;

                    if pressed && current_zone != HitZone::None {
                        new_dragging = Some(current_zone);
                    }

                    if down && let Some(zone) = new_dragging {
                        rect = zone.apply_drag(rect, delta, full);
                        ctx.request_repaint();
                    }

                    if released {
                        new_dragging = None;
                    }

                    self.mode = Mode::Selected {
                        rect,
                        dragging: new_dragging,
                    };

                    // 显示尺寸
                    draw_size_label(&painter, ctx, rect);

                    // 绘制调整手柄
                    draw_resize_handles(&painter, rect);
                }

                // 工具栏（只在不拖拽时响应）
                if dragging.is_none() {
                    match toolbar.show(ui, toolbar_rect) {
                        ToolbarAction::Cancel => {
                            self.cancel_overlay(ctx);
                        }
                        ToolbarAction::Confirm => {
                            let rect_px = points_rect_to_px(ctx, rect);
                            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                            self.mode = Mode::PendingCapture {
                                rect_px,
                                rect_points: rect,
                                hidden_at: Instant::now(),
                            };
                            ctx.request_repaint();
                        }
                        ToolbarAction::Arrow => {
                            // 切换箭头模式
                            self.arrow_mode_active = !self.arrow_mode_active;
                            // 同时显示/隐藏面板
                            self.show_arrow_panel = self.arrow_mode_active;
                            if !self.show_arrow_panel {
                                self.arrow_panel.popup_state = PopupState::None;
                            }
                        }
                        ToolbarAction::None => {}
                    }

                    // 显示箭头工具面板
                    if self.show_arrow_panel {
                        self.arrow_panel.show(ui, arrow_panel_rect, full);

                        // 点击面板和工具栏外部时关闭面板（但不退出箭头模式）
                        if let Some(click_pos) = pos
                            && pressed
                            && !arrow_panel_rect.contains(click_pos)
                            && !toolbar_rect.contains(click_pos)
                            && self.arrow_panel.popup_state == PopupState::None
                            && !rect.contains(click_pos)
                        {
                            self.show_arrow_panel = false;
                        }
                    }
                }

                // 箭头模式下的光标
                if self.arrow_mode_active
                    && let Some(mouse_pos) = pos
                    && rect.contains(mouse_pos)
                    && !in_ui_area
                {
                    let cursor = match self.arrow_drawer.state {
                        DrawState::Drawing => CursorIcon::Crosshair,
                        DrawState::Selected(_) => {
                            if let Some(hit) = self.arrow_drawer.hit_endpoint(mouse_pos) {
                                match hit {
                                    DrawState::MovingStart(_) | DrawState::MovingEnd(_) => {
                                        CursorIcon::Grab
                                    }
                                    DrawState::MovingWhole(_) => CursorIcon::Move,
                                    _ => CursorIcon::Crosshair,
                                }
                            } else {
                                CursorIcon::Crosshair
                            }
                        }
                        DrawState::MovingStart(_)
                        | DrawState::MovingEnd(_)
                        | DrawState::MovingWhole(_) => CursorIcon::Grabbing,
                        DrawState::Idle => CursorIcon::Crosshair,
                    };
                    ctx.set_cursor_icon(cursor);
                }
            });
    }

    /// 处理箭头模式下的鼠标输入
    fn handle_arrow_input(
        &mut self,
        ctx: &egui::Context,
        pos: egui::Pos2,
        pressed: bool,
        down: bool,
        released: bool,
    ) {
        match self.arrow_drawer.state {
            DrawState::Idle => {
                if pressed {
                    // 先尝试选中已有箭头
                    if !self.arrow_drawer.try_select(pos) {
                        // 没选中则开始绘制新箭头
                        self.arrow_drawer.start_drawing(pos);
                    }
                }
            }
            DrawState::Drawing => {
                if released {
                    self.arrow_drawer
                        .finish_drawing(pos, &self.arrow_panel.config);
                }
                ctx.request_repaint();
            }
            DrawState::Selected(_) => {
                if pressed {
                    // 检查是否点击了端点
                    if let Some(hit) = self.arrow_drawer.hit_endpoint(pos) {
                        self.arrow_drawer.start_move(pos, hit);
                    } else if !self.arrow_drawer.try_select(pos) {
                        // 点击空白处，开始绘制新箭头
                        self.arrow_drawer.start_drawing(pos);
                    }
                }
            }
            DrawState::MovingStart(_) | DrawState::MovingEnd(_) | DrawState::MovingWhole(_) => {
                if down {
                    self.arrow_drawer.update_move(pos);
                    ctx.request_repaint();
                }
                if released {
                    self.arrow_drawer.finish_move();
                }
            }
        }
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
