use super::config::ArrowConfig;
use super::types::{ArrowType, LineStyle, PRESET_COLORS};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

/// 弹出面板的状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopupState {
    #[default]
    None,
    ArrowType,   // 显示箭头类型选择
    LineStyle,   // 显示线段类型选择
    ColorPicker, // 显示颜色/大小选择器
}

/// Arrow 工具面板
pub struct ArrowToolPanel {
    pub config: ArrowConfig,
    pub popup_state: PopupState,
    pub panel_rect: Option<Rect>,
}

impl Default for ArrowToolPanel {
    fn default() -> Self {
        Self {
            config: ArrowConfig::default(),
            popup_state: PopupState::None,
            panel_rect: None,
        }
    }
}

impl ArrowToolPanel {
    /// 计算面板位置
    pub fn calc_panel_rect(&self, toolbar_rect: Rect, screen: Rect) -> Rect {
        let width = 400.0;
        let height = 44.0;

        // 默认放工具栏下方
        let mut center = Pos2::new(
            toolbar_rect.center().x,
            toolbar_rect.max.y + 8.0 + height / 2.0,
        );

        // 如果超出底部，放上方
        if center.y + height / 2.0 > screen.max.y - 8.0 {
            center.y = toolbar_rect.min.y - 8.0 - height / 2.0;
        }

        // clamp 到屏幕内
        center.x = center.x.clamp(
            screen.min.x + width / 2.0 + 8.0,
            screen.max.x - width / 2.0 - 8.0,
        );

        Rect::from_center_size(center, Vec2::new(width, height))
    }

    /// 计算弹出选择框的位置
    fn calc_popup_rect(
        &self,
        anchor_rect: Rect,
        popup_width: f32,
        popup_height: f32,
        screen: Rect,
    ) -> Rect {
        let mut pos = Pos2::new(
            anchor_rect.center().x - popup_width / 2.0,
            anchor_rect.max.y + 4.0,
        );

        // 如果超出底部，放上方
        if pos.y + popup_height > screen.max.y - 8.0 {
            pos.y = anchor_rect.min.y - popup_height - 4.0;
        }

        // clamp 到屏幕内
        pos.x = pos.x.clamp(8.0, screen.max.x - popup_width - 8.0);

        Rect::from_min_size(pos, Vec2::new(popup_width, popup_height))
    }

    /// 绘制箭头图标
    fn draw_arrow_icon(&self, painter: &egui::Painter, rect: Rect, arrow_type: ArrowType) {
        let color = Color32::GRAY;
        let stroke = Stroke::new(2.0, color);

        let left = Pos2::new(rect.min.x, rect.center().y);
        let right = Pos2::new(rect.max.x - 8.0, rect.center().y);

        painter.line_segment([left, right], stroke);
        match arrow_type {
            ArrowType::Single => {
                self.draw_arrow_head(painter, right, 1.0, color, false);
            }
            ArrowType::Double => {
                self.draw_arrow_head(painter, right, 1.0, color, false);
                self.draw_arrow_head(painter, left, -1.0, color, false);
            }
            ArrowType::Hollow => {
                self.draw_arrow_head(painter, right, 1.0, color, false);
            }
            ArrowType::Filled => {
                self.draw_arrow_head(painter, right, 1.0, color, true);
            }
        }
    }

    /// 绘制箭头头部
    fn draw_arrow_head(
        &self,
        painter: &egui::Painter,
        pos: Pos2,
        dir: f32,
        color: Color32,
        filled: bool,
    ) {
        let size = 6.0;
        let points = vec![
            pos,
            Pos2::new(pos.x - size * dir, pos.y - size * 0.5),
            Pos2::new(pos.x - size * dir, pos.y + size * 0.5),
        ];

        if filled {
            painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
        } else {
            painter.add(egui::Shape::closed_line(points, Stroke::new(1.5, color)));
        }
    }

    /// 绘制箭头类型按钮
    fn draw_arrow_type_button(&self, ui: &mut egui::Ui, rect: Rect) -> bool {
        let response = ui.allocate_rect(rect, Sense::click());
        let painter = ui.painter();

        let bg_color = if response.hovered() {
            Color32::from_rgb(70, 70, 70)
        } else {
            Color32::from_rgb(60, 60, 60)
        };

        painter.rect_filled(rect, 4.0, bg_color);
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
            StrokeKind::Inside,
        );

        // 绘制箭头图标（留出下拉箭头空间）
        let icon_rect = Rect::from_min_max(
            rect.min + Vec2::new(6.0, 6.0),
            Pos2::new(rect.max.x - 16.0, rect.max.y - 6.0),
        );
        self.draw_arrow_icon(painter, icon_rect, self.config.arrow_type);

        // 绘制下拉三角形
        Self::draw_dropdown_triangle(painter, Pos2::new(rect.max.x - 8.0, rect.center().y));

        response.clicked()
    }

    /// 绘制线段类型按钮
    fn draw_line_style_button(&self, ui: &mut egui::Ui, rect: Rect) -> bool {
        let response = ui.allocate_rect(rect, Sense::click());
        let painter = ui.painter();

        let bg_color = if response.hovered() {
            Color32::from_rgb(70, 70, 70)
        } else {
            Color32::from_rgb(60, 60, 60)
        };

        painter.rect_filled(rect, 4.0, bg_color);
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
            StrokeKind::Inside,
        );

        // 绘制线段图标（留出下拉箭头空间）
        let icon_rect = Rect::from_min_max(
            rect.min + Vec2::new(6.0, 6.0),
            Pos2::new(rect.max.x - 16.0, rect.max.y - 6.0),
        );
        self.draw_line_icon(painter, icon_rect, self.config.line_style);

        // 绘制下拉三角形
        Self::draw_dropdown_triangle(painter, Pos2::new(rect.max.x - 8.0, rect.center().y));

        response.clicked()
    }

    /// 绘制下拉三角形
    fn draw_dropdown_triangle(painter: &egui::Painter, center: Pos2) {
        let size = 4.0;
        let points = vec![
            Pos2::new(center.x - size, center.y - size * 0.5),
            Pos2::new(center.x + size, center.y - size * 0.5),
            Pos2::new(center.x, center.y + size * 0.5),
        ];
        painter.add(egui::Shape::convex_polygon(
            points,
            Color32::GRAY,
            Stroke::NONE,
        ));
    }

    /// 绘制颜色/大小按钮
    fn draw_color_size_button(&self, ui: &mut egui::Ui, rect: Rect) -> bool {
        let response = ui.allocate_rect(rect, Sense::click());
        let painter = ui.painter();

        // 蓝色高亮背景
        painter.rect_filled(rect, 4.0, Color32::from_rgb(0, 122, 255));

        // 绘制颜色点
        let dot_pos = Pos2::new(rect.min.x + 16.0, rect.center().y);
        painter.circle_filled(dot_pos, 6.0, self.config.color);
        painter.circle_stroke(dot_pos, 6.0, Stroke::new(1.0, Color32::WHITE));

        // 绘制大小文字
        let text_pos = Pos2::new(rect.center().x + 8.0, rect.center().y);
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            format!("{}", self.config.stroke_width as i32),
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );

        response.clicked()
    }

    /// 绘制预设颜色按钮
    fn draw_color_preset_button(&self, ui: &mut egui::Ui, rect: Rect, color: Color32) -> bool {
        let response = ui.allocate_rect(rect, Sense::click());
        let painter = ui.painter();

        // 颜色方块
        painter.rect_filled(rect, 2.0, color);

        // 如果是白色，添加边框
        if color == Color32::WHITE {
            painter.rect_stroke(
                rect,
                2.0,
                Stroke::new(1.0, Color32::GRAY),
                StrokeKind::Inside,
            );
        }

        // 如果当前选中，添加选中边框
        if self.config.color == color {
            painter.rect_stroke(
                rect.expand(2.0),
                2.0,
                Stroke::new(2.0, Color32::WHITE),
                StrokeKind::Inside,
            );
        }

        response.clicked()
    }

    /// 绘制线段图标
    fn draw_line_icon(&self, painter: &egui::Painter, rect: Rect, line_style: LineStyle) {
        let color = Color32::GRAY;
        let y = rect.center().y;
        let left = rect.min.x;
        let right = rect.max.x - 8.0;

        match line_style {
            LineStyle::Solid => {
                painter.line_segment(
                    [Pos2::new(left, y), Pos2::new(right, y)],
                    Stroke::new(2.0, color),
                );
            }
            LineStyle::Dashed => {
                let dash_len = 6.0;
                let gap_len = 4.0;
                let mut x = left;
                while x < right {
                    let end_x = (x + dash_len).min(right);
                    painter.line_segment(
                        [Pos2::new(x, y), Pos2::new(end_x, y)],
                        Stroke::new(2.0, color),
                    );
                    x += dash_len + gap_len;
                }
            }
            LineStyle::Dotted => {
                let dot_gap = 4.0;
                let mut x = left;
                while x < right {
                    painter.circle_filled(Pos2::new(x, y), 1.5, color);
                    x += dot_gap;
                }
            }
            LineStyle::DashDot => {
                let mut x = left;
                let mut is_dash = true;
                while x < right {
                    if is_dash {
                        let end_x = (x + 6.0).min(right);
                        painter.line_segment(
                            [Pos2::new(x, y), Pos2::new(end_x, y)],
                            Stroke::new(2.0, color),
                        );
                        x += 8.0;
                    } else {
                        painter.circle_filled(Pos2::new(x, y), 1.5, color);
                        x += 4.0;
                    }
                    is_dash = !is_dash;
                }
            }
            LineStyle::DashDotDot => {
                let mut x = left;
                let mut state = 0;
                while x < right {
                    match state {
                        0 => {
                            let end_x = (x + 6.0).min(right);
                            painter.line_segment(
                                [Pos2::new(x, y), Pos2::new(end_x, y)],
                                Stroke::new(2.0, color),
                            );
                            x += 8.0;
                        }
                        1 | 2 => {
                            painter.circle_filled(Pos2::new(x, y), 1.5, color);
                            x += 4.0;
                        }
                        _ => {}
                    }
                    state = (state + 1) % 3;
                }
            }
        }
    }

    /// 绘制箭头类型弹出菜单
    fn draw_arrow_type_popup(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let item_height = 32.0;
        let mut y = rect.min.y + 4.0;

        // 第一步：收集所有响应
        let mut responses = Vec::new();
        for arrow_type in ArrowType::all() {
            let item_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + 4.0, y),
                Vec2::new(rect.width() - 8.0, item_height),
            );
            let response = ui.allocate_rect(item_rect, Sense::click());
            responses.push((*arrow_type, item_rect, response));
            y += item_height;
        }

        // 第二步：绘制
        let painter = ui.painter();

        // 背景
        painter.rect_filled(rect, 6.0, Color32::from_rgb(45, 45, 45));
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0, Color32::from_rgb(80, 80, 80)),
            StrokeKind::Inside,
        );

        let mut clicked_type = None;
        for (arrow_type, item_rect, response) in &responses {
            let bg_color = if *arrow_type == self.config.arrow_type {
                Color32::from_rgb(0, 122, 255)
            } else if response.hovered() {
                Color32::from_rgb(60, 60, 60)
            } else {
                Color32::TRANSPARENT
            };

            painter.rect_filled(*item_rect, 4.0, bg_color);

            // 只绘制箭头图标（居中）
            let icon_rect = Rect::from_center_size(
                item_rect.center(),
                Vec2::new(item_rect.width() - 16.0, item_height - 12.0),
            );
            self.draw_arrow_icon(painter, icon_rect, *arrow_type);

            if response.clicked() {
                clicked_type = Some(*arrow_type);
            }
        }

        // 第三步：更新状态
        if let Some(arrow_type) = clicked_type {
            self.config.arrow_type = arrow_type;
            self.popup_state = PopupState::None;
        }
    }

    /// 绘制线段类型弹出菜单
    fn draw_line_style_popup(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let item_height = 28.0;
        let mut y = rect.min.y + 4.0;

        // 第一步：收集所有响应
        let mut responses = Vec::new();
        for line_style in LineStyle::all() {
            let item_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + 4.0, y),
                Vec2::new(rect.width() - 8.0, item_height),
            );
            let response = ui.allocate_rect(item_rect, Sense::click());
            responses.push((*line_style, item_rect, response));
            y += item_height;
        }

        // 第二步：绘制
        let painter = ui.painter();

        // 背景
        painter.rect_filled(rect, 6.0, Color32::from_rgb(45, 45, 45));
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0, Color32::from_rgb(80, 80, 80)),
            StrokeKind::Inside,
        );

        let mut clicked_style = None;
        for (line_style, item_rect, response) in &responses {
            let bg_color = if *line_style == self.config.line_style {
                Color32::from_rgb(0, 122, 255)
            } else if response.hovered() {
                Color32::from_rgb(60, 60, 60)
            } else {
                Color32::TRANSPARENT
            };

            painter.rect_filled(*item_rect, 4.0, bg_color);

            // 只绘制线段图标（居中）
            let icon_rect = Rect::from_center_size(
                item_rect.center(),
                Vec2::new(item_rect.width() - 16.0, item_height - 10.0),
            );
            self.draw_line_icon(painter, icon_rect, *line_style);

            if response.clicked() {
                clicked_style = Some(*line_style);
            }
        }

        // 第三步：更新状态
        if let Some(line_style) = clicked_style {
            self.config.line_style = line_style;
            self.popup_state = PopupState::None;
        }
    }

    /// 绘制颜色选择器弹出菜单
    fn draw_color_picker_popup(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let padding = 10.0;
        let mut y = rect.min.y + padding;

        // 颜色选择区域 - 色相/饱和度方块
        let color_box_size = 120.0;
        let color_box_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + padding, y),
            Vec2::new(color_box_size, color_box_size),
        );
        let color_box_response = ui.allocate_rect(color_box_rect, Sense::click_and_drag());

        // 亮度滑块
        let brightness_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + padding + color_box_size + 8.0, y),
            Vec2::new(16.0, color_box_size),
        );
        let brightness_response = ui.allocate_rect(brightness_rect, Sense::click_and_drag());

        y += color_box_size + 12.0;

        // 大小滑块
        let slider_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + padding, y + 16.0),
            Vec2::new(rect.width() - padding * 2.0, 20.0),
        );
        let slider_response = ui.allocate_rect(slider_rect, Sense::click_and_drag());

        // 第二步：绘制
        let painter = ui.painter();

        // 背景
        painter.rect_filled(rect, 6.0, Color32::from_rgb(45, 45, 45));
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0, Color32::from_rgb(80, 80, 80)),
            StrokeKind::Inside,
        );

        // 绘制颜色选择方块（简化版：从红到蓝的渐变 + 从白到黑的渐变）
        self.draw_color_picker_box(painter, color_box_rect);

        // 绘制亮度滑块
        self.draw_brightness_slider(painter, brightness_rect);

        // 当前颜色预览
        let preview_rect = Rect::from_min_size(
            Pos2::new(brightness_rect.max.x + 8.0, color_box_rect.min.y),
            Vec2::new(24.0, 24.0),
        );
        painter.rect_filled(preview_rect, 2.0, self.config.color);
        painter.rect_stroke(
            preview_rect,
            2.0,
            Stroke::new(1.0, Color32::WHITE),
            StrokeKind::Inside,
        );

        // 大小滑块标签
        painter.text(
            Pos2::new(rect.min.x + padding, y),
            egui::Align2::LEFT_CENTER,
            format!("{}px", self.config.stroke_width as i32),
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );

        // 大小滑块轨道
        let track_rect = Rect::from_min_size(
            Pos2::new(slider_rect.min.x, slider_rect.center().y - 3.0),
            Vec2::new(slider_rect.width(), 6.0),
        );
        painter.rect_filled(track_rect, 3.0, Color32::from_rgb(80, 80, 80));

        // 滑块位置（1-48 范围）
        let max_size = 48.0;
        let ratio = (self.config.stroke_width - 1.0) / (max_size - 1.0);
        let thumb_x = slider_rect.min.x + ratio * slider_rect.width();
        let thumb_pos = Pos2::new(thumb_x, slider_rect.center().y);
        painter.circle_filled(thumb_pos, 8.0, Color32::WHITE);

        // 第三步：处理交互
        if (color_box_response.clicked() || color_box_response.dragged())
            && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
        {
            let rel_x = ((pos.x - color_box_rect.min.x) / color_box_rect.width()).clamp(0.0, 1.0);
            let rel_y = ((pos.y - color_box_rect.min.y) / color_box_rect.height()).clamp(0.0, 1.0);
            // 简化颜色选择：x = 色相，y = 饱和度
            let hue = rel_x * 360.0;
            let saturation = 1.0 - rel_y;
            self.config.color = Self::hsv_to_rgb(hue, saturation, 1.0);
        }

        if (brightness_response.clicked() || brightness_response.dragged())
            && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
        {
            let rel_y =
                ((pos.y - brightness_rect.min.y) / brightness_rect.height()).clamp(0.0, 1.0);
            let value = 1.0 - rel_y;
            // 调整当前颜色的亮度
            let (h, s, _) = Self::rgb_to_hsv(self.config.color);
            self.config.color = Self::hsv_to_rgb(h, s, value);
        }

        if (slider_response.clicked() || slider_response.dragged())
            && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
        {
            let new_ratio = ((pos.x - slider_rect.min.x) / slider_rect.width()).clamp(0.0, 1.0);
            self.config.stroke_width = (1.0 + new_ratio * (max_size - 1.0)).round();
        }
    }

    /// 绘制颜色选择方块
    fn draw_color_picker_box(&self, painter: &egui::Painter, rect: Rect) {
        let steps = 20;
        let step_w = rect.width() / steps as f32;
        let step_h = rect.height() / steps as f32;

        for ix in 0..steps {
            for iy in 0..steps {
                let hue = (ix as f32 / steps as f32) * 360.0;
                let sat = 1.0 - (iy as f32 / steps as f32);
                let color = Self::hsv_to_rgb(hue, sat, 1.0);

                let cell_rect = Rect::from_min_size(
                    Pos2::new(
                        rect.min.x + ix as f32 * step_w,
                        rect.min.y + iy as f32 * step_h,
                    ),
                    Vec2::new(step_w + 1.0, step_h + 1.0),
                );
                painter.rect_filled(cell_rect, 0.0, color);
            }
        }

        // 边框
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, Color32::from_rgb(80, 80, 80)),
            StrokeKind::Inside,
        );
    }

    /// 绘制亮度滑块
    fn draw_brightness_slider(&self, painter: &egui::Painter, rect: Rect) {
        let steps = 20;
        let step_h = rect.height() / steps as f32;

        for i in 0..steps {
            let value = 1.0 - (i as f32 / steps as f32);
            let gray = (value * 255.0) as u8;
            let color = Color32::from_rgb(gray, gray, gray);

            let cell_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, rect.min.y + i as f32 * step_h),
                Vec2::new(rect.width(), step_h + 1.0),
            );
            painter.rect_filled(cell_rect, 0.0, color);
        }

        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, Color32::from_rgb(80, 80, 80)),
            StrokeKind::Inside,
        );
    }

    /// HSV 转 RGB
    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color32 {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Color32::from_rgb(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }

    /// RGB 转 HSV
    fn rgb_to_hsv(color: Color32) -> (f32, f32, f32) {
        let r = color.r() as f32 / 255.0;
        let g = color.g() as f32 / 255.0;
        let b = color.b() as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };
        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        (h, s, v)
    }

    /// 检查点击是否在面板外部
    pub fn is_click_outside(&self, pos: Pos2) -> bool {
        if let Some(panel_rect) = self.panel_rect {
            !panel_rect.contains(pos)
        } else {
            true
        }
    }

    /// 绘制主面板
    pub fn show(&mut self, ui: &mut egui::Ui, panel_rect: Rect, screen: Rect) -> bool {
        let painter = ui.painter();
        self.panel_rect = Some(panel_rect);

        // 面板背景
        painter.rect_filled(panel_rect, 6.0, Color32::from_rgb(50, 50, 50));
        painter.rect_stroke(
            panel_rect,
            6.0,
            Stroke::new(1.5, Color32::from_rgb(80, 80, 80)),
            StrokeKind::Inside,
        );

        let padding = 10.0;
        let btn_height = panel_rect.height() - padding * 2.0;
        let mut x = panel_rect.min.x + padding;
        let y = panel_rect.min.y + padding;

        // 1. 箭头类型按钮（带下拉箭头）
        let arrow_btn_width = 60.0;
        let arrow_btn_rect =
            Rect::from_min_size(Pos2::new(x, y), Vec2::new(arrow_btn_width, btn_height));
        if self.draw_arrow_type_button(ui, arrow_btn_rect) {
            self.popup_state = if self.popup_state == PopupState::ArrowType {
                PopupState::None
            } else {
                PopupState::ArrowType
            };
        }
        x += arrow_btn_width + padding;

        // 2. 线段类型按钮（带下拉箭头）
        let line_btn_width = 60.0;
        let line_btn_rect =
            Rect::from_min_size(Pos2::new(x, y), Vec2::new(line_btn_width, btn_height));
        if self.draw_line_style_button(ui, line_btn_rect) {
            self.popup_state = if self.popup_state == PopupState::LineStyle {
                PopupState::None
            } else {
                PopupState::LineStyle
            };
        }
        x += line_btn_width + padding;

        // 3. 颜色/大小按钮
        let color_btn_width = 60.0;
        let color_btn_rect =
            Rect::from_min_size(Pos2::new(x, y), Vec2::new(color_btn_width, btn_height));
        if self.draw_color_size_button(ui, color_btn_rect) {
            self.popup_state = if self.popup_state == PopupState::ColorPicker {
                PopupState::None
            } else {
                PopupState::ColorPicker
            };
        }
        x += color_btn_width + padding;

        // 4. 快速预设颜色按钮
        let preset_size = btn_height;
        for (color, _name) in PRESET_COLORS {
            let preset_rect =
                Rect::from_min_size(Pos2::new(x, y), Vec2::new(preset_size, preset_size));
            if self.draw_color_preset_button(ui, preset_rect, *color) {
                self.config.color = *color;
            }
            x += preset_size + 4.0;
        }

        // 绘制弹出面板
        match self.popup_state {
            PopupState::ArrowType => {
                // 4 items * 32px + 8px padding
                let popup_rect = self.calc_popup_rect(arrow_btn_rect, 100.0, 136.0, screen);
                self.draw_arrow_type_popup(ui, popup_rect);
            }
            PopupState::LineStyle => {
                // 5 items * 28px + 8px padding
                let popup_rect = self.calc_popup_rect(line_btn_rect, 100.0, 148.0, screen);
                self.draw_line_style_popup(ui, popup_rect);
            }
            PopupState::ColorPicker => {
                // 颜色方块 120 + 亮度条 16 + 间距 + padding
                let popup_rect = self.calc_popup_rect(color_btn_rect, 190.0, 190.0, screen);
                self.draw_color_picker_popup(ui, popup_rect);
            }
            PopupState::None => {}
        }

        false
    }
}
