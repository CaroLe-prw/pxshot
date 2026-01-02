use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use image::{ImageBuffer, Rgba};
use imageproc::drawing::draw_antialiased_polygon_mut;
use imageproc::pixelops::interpolate;
use imageproc::point::Point;

use super::config::ArrowConfig;
use super::types::{ArrowType, LineStyle};

/// RgbaImage 类型别名
pub type RgbaImage = ImageBuffer<Rgba<u8>, Vec<u8>>;

/// 单个箭头实例
#[derive(Debug, Clone)]
pub struct Arrow {
    pub start: Pos2,
    pub end: Pos2,
    pub arrow_type: ArrowType,
    pub line_style: LineStyle,
    pub color: Color32,
    pub stroke_width: f32,
}

impl Arrow {
    /// 从配置创建新箭头
    pub fn new(start: Pos2, end: Pos2, config: &ArrowConfig) -> Self {
        Self {
            start,
            end,
            arrow_type: config.arrow_type,
            line_style: config.line_style,
            color: config.color,
            stroke_width: config.stroke_width,
        }
    }

    /// 获取箭头的包围盒
    pub fn bounding_rect(&self) -> Rect {
        let min_x = self.start.x.min(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_x = self.start.x.max(self.end.x);
        let max_y = self.start.y.max(self.end.y);
        Rect::from_min_max(
            Pos2::new(min_x - self.stroke_width, min_y - self.stroke_width),
            Pos2::new(max_x + self.stroke_width, max_y + self.stroke_width),
        )
    }

    /// 检测点是否在箭头附近（用于选中）
    pub fn hit_test(&self, pos: Pos2, tolerance: f32) -> bool {
        let dist = Self::point_to_line_distance(pos, self.start, self.end);
        dist <= self.stroke_width / 2.0 + tolerance
    }

    /// 点到线段的距离
    fn point_to_line_distance(point: Pos2, line_start: Pos2, line_end: Pos2) -> f32 {
        let line = line_end - line_start;
        let len_sq = line.length_sq();

        if len_sq == 0.0 {
            return (point - line_start).length();
        }

        let t = ((point - line_start).dot(line) / len_sq).clamp(0.0, 1.0);
        let projection = line_start + line * t;
        (point - projection).length()
    }

    /// 绘制箭头
    pub fn draw(&self, painter: &egui::Painter) {
        let stroke = Stroke::new(self.stroke_width, self.color);
        let dir = (self.end - self.start).normalized();
        let head_size = self.stroke_width * 3.0;

        match self.arrow_type {
            ArrowType::Single => {
                // 直线箭头：线条到三角形底部 + 填充三角形
                let line_end = self.end - dir * head_size;
                self.draw_line(painter, self.start, line_end, stroke);
                self.draw_v_arrow_head(painter, self.end, dir, head_size);
            }
            ArrowType::Double => {
                // 双向箭头：两端都有填充三角形
                let line_start = self.start + dir * head_size;
                let line_end = self.end - dir * head_size;
                self.draw_line(painter, line_start, line_end, stroke);
                self.draw_v_arrow_head(painter, self.end, dir, head_size);
                self.draw_v_arrow_head(painter, self.start, -dir, head_size);
            }
            ArrowType::Hollow => {
                // 空心箭头：整个形状只有轮廓
                self.draw_hollow_arrow(painter, dir, head_size);
            }
            ArrowType::Filled => {
                // 实心箭头：渐变粗线条 + 实心三角形
                let back = self.end - dir * head_size;
                self.draw_tapered_line(painter, self.start, back, dir, head_size);
                self.draw_filled_triangle(painter, self.end, dir, head_size);
            }
        }
    }

    /// 绘制 V 形箭头尖端（用于直线和双向箭头）- 使用填充三角形
    fn draw_v_arrow_head(&self, painter: &egui::Painter, tip: Pos2, dir: Vec2, size: f32) {
        let perpendicular = Vec2::new(-dir.y, dir.x);
        // 箭头尖端的小填充三角形
        let back = tip - dir * size;
        let left = back + perpendicular * size * 0.35;
        let right = back - perpendicular * size * 0.35;

        let points = vec![tip, left, right];
        painter.add(egui::Shape::convex_polygon(
            points,
            self.color,
            Stroke::NONE,
        ));
    }

    /// 绘制整个空心箭头（从细到粗的轮廓 + 空心三角形）
    fn draw_hollow_arrow(&self, painter: &egui::Painter, dir: Vec2, head_size: f32) {
        let perpendicular = Vec2::new(-dir.y, dir.x);
        let stroke = Stroke::new(self.stroke_width * 0.4, self.color);

        // 计算关键点
        let start_width = self.stroke_width * 0.15;
        let end_width = head_size * 0.5;
        let back = self.end - dir * head_size;

        // 线条部分的四个角点
        let p1 = self.start + perpendicular * start_width; // 起点上
        let p2 = self.start - perpendicular * start_width; // 起点下
        let p3 = back - perpendicular * end_width; // 底部下
        let p4 = back + perpendicular * end_width; // 底部上

        // 三角形的三个顶点
        let tip = self.end;
        let tri_left = back + perpendicular * head_size * 0.5;
        let tri_right = back - perpendicular * head_size * 0.5;

        // 绘制外轮廓（从起点上方 -> 底部上方 -> 三角形左 -> 尖端 -> 三角形右 -> 底部下方 -> 起点下方 -> 起点上方）
        painter.line_segment([p1, p4], stroke); // 上边线
        painter.line_segment([p4, tri_left], stroke); // 到三角形左
        painter.line_segment([tri_left, tip], stroke); // 三角形左边
        painter.line_segment([tip, tri_right], stroke); // 三角形右边
        painter.line_segment([tri_right, p3], stroke); // 从三角形右
        painter.line_segment([p3, p2], stroke); // 下边线
        painter.line_segment([p2, p1], stroke); // 起点封口
    }

    /// 绘制实心三角形
    fn draw_filled_triangle(&self, painter: &egui::Painter, tip: Pos2, dir: Vec2, size: f32) {
        let perpendicular = Vec2::new(-dir.y, dir.x);
        let back = tip - dir * size;
        let left = back + perpendicular * size * 0.5;
        let right = back - perpendicular * size * 0.5;

        let points = vec![tip, left, right];
        painter.add(egui::Shape::convex_polygon(
            points,
            self.color,
            Stroke::NONE,
        ));
    }

    /// 绘制渐变粗细的线条（从细到粗）
    fn draw_tapered_line(
        &self,
        painter: &egui::Painter,
        start: Pos2,
        end: Pos2,
        dir: Vec2,
        head_size: f32,
    ) {
        let perpendicular = Vec2::new(-dir.y, dir.x);
        let start_width = self.stroke_width * 0.3;
        let end_width = head_size * 0.5;

        // 创建梯形形状
        let p1 = start + perpendicular * start_width * 0.5;
        let p2 = start - perpendicular * start_width * 0.5;
        let p3 = end - perpendicular * end_width;
        let p4 = end + perpendicular * end_width;

        let points = vec![p1, p4, p3, p2];
        painter.add(egui::Shape::convex_polygon(
            points,
            self.color,
            Stroke::NONE,
        ));
    }

    /// 根据线段类型绘制线条
    fn draw_line(&self, painter: &egui::Painter, start: Pos2, end: Pos2, stroke: Stroke) {
        match self.line_style {
            LineStyle::Solid => {
                painter.line_segment([start, end], stroke);
            }
            LineStyle::Dashed => {
                self.draw_dashed_line(painter, start, end, stroke, 12.0, 6.0);
            }
            LineStyle::Dotted => {
                self.draw_dotted_line(painter, start, end, stroke);
            }
            LineStyle::DashDot => {
                self.draw_dash_dot_line(painter, start, end, stroke);
            }
            LineStyle::DashDotDot => {
                self.draw_dash_dot_dot_line(painter, start, end, stroke);
            }
        }
    }

    /// 绘制虚线
    fn draw_dashed_line(
        &self,
        painter: &egui::Painter,
        start: Pos2,
        end: Pos2,
        stroke: Stroke,
        dash_len: f32,
        gap_len: f32,
    ) {
        let dir = (end - start).normalized();
        let total_len = (end - start).length();
        let mut pos = 0.0;
        let mut is_dash = true;

        while pos < total_len {
            if is_dash {
                let seg_end = (pos + dash_len).min(total_len);
                painter.line_segment([start + dir * pos, start + dir * seg_end], stroke);
                pos = seg_end;
            } else {
                pos += gap_len;
            }
            is_dash = !is_dash;
        }
    }

    /// 绘制点线
    fn draw_dotted_line(&self, painter: &egui::Painter, start: Pos2, end: Pos2, stroke: Stroke) {
        let dir = (end - start).normalized();
        let total_len = (end - start).length();
        let dot_gap = stroke.width * 2.0;
        let mut pos = 0.0;

        while pos < total_len {
            painter.circle_filled(start + dir * pos, stroke.width / 2.0, stroke.color);
            pos += dot_gap;
        }
    }

    /// 绘制点划线
    fn draw_dash_dot_line(&self, painter: &egui::Painter, start: Pos2, end: Pos2, stroke: Stroke) {
        let dir = (end - start).normalized();
        let total_len = (end - start).length();
        let dash_len = 12.0;
        let gap_len = 4.0;
        let mut pos = 0.0;
        let mut state = 0; // 0=dash, 1=gap, 2=dot, 3=gap

        while pos < total_len {
            match state {
                0 => {
                    let seg_end = (pos + dash_len).min(total_len);
                    painter.line_segment([start + dir * pos, start + dir * seg_end], stroke);
                    pos = seg_end;
                }
                1 | 3 => {
                    pos += gap_len;
                }
                2 => {
                    painter.circle_filled(start + dir * pos, stroke.width / 2.0, stroke.color);
                    pos += stroke.width;
                }
                _ => {}
            }
            state = (state + 1) % 4;
        }
    }

    /// 绘制双点划线
    fn draw_dash_dot_dot_line(
        &self,
        painter: &egui::Painter,
        start: Pos2,
        end: Pos2,
        stroke: Stroke,
    ) {
        let dir = (end - start).normalized();
        let total_len = (end - start).length();
        let dash_len = 12.0;
        let gap_len = 4.0;
        let mut pos = 0.0;
        let mut state = 0; // 0=dash, 1=gap, 2=dot, 3=gap, 4=dot, 5=gap

        while pos < total_len {
            match state {
                0 => {
                    let seg_end = (pos + dash_len).min(total_len);
                    painter.line_segment([start + dir * pos, start + dir * seg_end], stroke);
                    pos = seg_end;
                }
                1 | 3 | 5 => {
                    pos += gap_len;
                }
                2 | 4 => {
                    painter.circle_filled(start + dir * pos, stroke.width / 2.0, stroke.color);
                    pos += stroke.width;
                }
                _ => {}
            }
            state = (state + 1) % 6;
        }
    }

    // ========== 渲染到图像的方法 ==========

    fn blend_rgba(line: Rgba<u8>, original: Rgba<u8>, weight: f32) -> Rgba<u8> {
        interpolate(line, original, weight.clamp(0.0, 1.0))
    }

    fn round_point(p: (f32, f32)) -> Point<i32> {
        Point::new(p.0.round() as i32, p.1.round() as i32)
    }

    fn draw_antialiased_circle(
        img: &mut RgbaImage,
        center: (f32, f32),
        radius: f32,
        color: Rgba<u8>,
    ) {
        if radius <= 0.0 {
            return;
        }

        let r = radius.max(0.5);
        let min_x = (center.0 - r - 1.0).floor() as i32;
        let max_x = (center.0 + r + 1.0).ceil() as i32;
        let min_y = (center.1 - r - 1.0).floor() as i32;
        let max_y = (center.1 + r + 1.0).ceil() as i32;

        let width = img.width() as i32;
        let height = img.height() as i32;

        for y in min_y..=max_y {
            if y < 0 || y >= height {
                continue;
            }
            for x in min_x..=max_x {
                if x < 0 || x >= width {
                    continue;
                }

                let dx = x as f32 + 0.5 - center.0;
                let dy = y as f32 + 0.5 - center.1;
                let dist = (dx * dx + dy * dy).sqrt();
                let delta = r - dist;

                if delta >= 0.5 {
                    img.put_pixel(x as u32, y as u32, color);
                } else if delta > -0.5 {
                    let weight = (delta + 0.5).clamp(0.0, 1.0);
                    let original = img.get_pixel(x as u32, y as u32);
                    let blended = Self::blend_rgba(color, *original, weight);
                    img.put_pixel(x as u32, y as u32, blended);
                }
            }
        }
    }

    fn draw_aa_line_with_width(
        img: &mut RgbaImage,
        start: (f32, f32),
        end: (f32, f32),
        width: f32,
        color: Rgba<u8>,
    ) {
        let stroke_width = width.max(1.0);
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let len = (dx * dx + dy * dy).sqrt();

        if len == 0.0 {
            Self::draw_antialiased_circle(img, start, stroke_width / 2.0, color);
            return;
        }

        // 计算包围线段的矩形，使用抗锯齿多边形绘制
        let perp = (-dy / len, dx / len);
        let half = stroke_width / 2.0;

        let p1 = (start.0 + perp.0 * half, start.1 + perp.1 * half);
        let p2 = (start.0 - perp.0 * half, start.1 - perp.1 * half);
        let p3 = (end.0 - perp.0 * half, end.1 - perp.1 * half);
        let p4 = (end.0 + perp.0 * half, end.1 + perp.1 * half);

        let points = [
            Self::round_point(p1),
            Self::round_point(p2),
            Self::round_point(p3),
            Self::round_point(p4),
        ];
        draw_antialiased_polygon_mut(img, &points, color, Self::blend_rgba);

        // 圆角端点
        Self::draw_antialiased_circle(img, start, half, color);
        Self::draw_antialiased_circle(img, end, half, color);
    }

    /// 将箭头渲染到图像上
    pub fn render_to_image(&self, img: &mut RgbaImage, offset_x: f32, offset_y: f32, ppp: f32) {
        let color = Rgba([
            self.color.r(),
            self.color.g(),
            self.color.b(),
            self.color.a(),
        ]);

        // 转换坐标
        let start = (
            (self.start.x - offset_x) * ppp,
            (self.start.y - offset_y) * ppp,
        );
        let end = ((self.end.x - offset_x) * ppp, (self.end.y - offset_y) * ppp);

        let stroke_width_px = self.stroke_width * ppp;
        let head_size = stroke_width_px * 3.0;

        let dir_x = end.0 - start.0;
        let dir_y = end.1 - start.1;
        let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
        if len == 0.0 {
            return;
        }
        let dir = (dir_x / len, dir_y / len);

        match self.arrow_type {
            ArrowType::Single => {
                // 直线箭头：线条到三角形底部 + 填充三角形
                let line_end = (end.0 - dir.0 * head_size, end.1 - dir.1 * head_size);
                self.render_line_to_image(img, start, line_end, color, stroke_width_px);
                Self::render_v_arrow_head(img, end, dir, head_size, stroke_width_px, color);
            }
            ArrowType::Double => {
                // 双向箭头：两端都有填充三角形
                let line_start = (start.0 + dir.0 * head_size, start.1 + dir.1 * head_size);
                let line_end = (end.0 - dir.0 * head_size, end.1 - dir.1 * head_size);
                self.render_line_to_image(img, line_start, line_end, color, stroke_width_px);
                Self::render_v_arrow_head(img, end, dir, head_size, stroke_width_px, color);
                Self::render_v_arrow_head(
                    img,
                    start,
                    (-dir.0, -dir.1),
                    head_size,
                    stroke_width_px,
                    color,
                );
            }
            ArrowType::Hollow => {
                // 空心箭头：整个形状只有轮廓
                Self::render_hollow_arrow(img, start, end, dir, stroke_width_px, head_size, color);
            }
            ArrowType::Filled => {
                // 实心箭头：渐变粗线条 + 实心三角形
                let back = (end.0 - dir.0 * head_size, end.1 - dir.1 * head_size);
                Self::render_tapered_line(img, start, back, dir, stroke_width_px, head_size, color);
                Self::render_filled_triangle(img, end, dir, head_size, color);
            }
        }
    }

    /// 渲染 V 形箭头尖端 - 使用填充三角形
    fn render_v_arrow_head(
        img: &mut RgbaImage,
        tip: (f32, f32),
        dir: (f32, f32),
        size: f32,
        _stroke_width: f32,
        color: Rgba<u8>,
    ) {
        let perpendicular = (-dir.1, dir.0);
        let back = (tip.0 - dir.0 * size, tip.1 - dir.1 * size);
        let left = (
            back.0 + perpendicular.0 * size * 0.35,
            back.1 + perpendicular.1 * size * 0.35,
        );
        let right = (
            back.0 - perpendicular.0 * size * 0.35,
            back.1 - perpendicular.1 * size * 0.35,
        );

        let points = [
            Self::round_point(tip),
            Self::round_point(left),
            Self::round_point(right),
        ];
        draw_antialiased_polygon_mut(img, &points, color, Self::blend_rgba);
    }

    /// 渲染整个空心箭头
    fn render_hollow_arrow(
        img: &mut RgbaImage,
        start: (f32, f32),
        end: (f32, f32),
        dir: (f32, f32),
        stroke_width: f32,
        head_size: f32,
        color: Rgba<u8>,
    ) {
        let perpendicular = (-dir.1, dir.0);
        let line_width = (stroke_width * 0.4).max(1.0);

        // 计算关键点
        let start_width = stroke_width * 0.15;
        let end_width = head_size * 0.5;
        let back = (end.0 - dir.0 * head_size, end.1 - dir.1 * head_size);

        // 线条部分的四个角点
        let p1 = (
            start.0 + perpendicular.0 * start_width,
            start.1 + perpendicular.1 * start_width,
        );
        let p2 = (
            start.0 - perpendicular.0 * start_width,
            start.1 - perpendicular.1 * start_width,
        );
        let p3 = (
            back.0 - perpendicular.0 * end_width,
            back.1 - perpendicular.1 * end_width,
        );
        let p4 = (
            back.0 + perpendicular.0 * end_width,
            back.1 + perpendicular.1 * end_width,
        );

        // 三角形的三个顶点
        let tip = end;
        let tri_left = (
            back.0 + perpendicular.0 * head_size * 0.5,
            back.1 + perpendicular.1 * head_size * 0.5,
        );
        let tri_right = (
            back.0 - perpendicular.0 * head_size * 0.5,
            back.1 - perpendicular.1 * head_size * 0.5,
        );

        // 绘制外轮廓
        Self::draw_aa_line_with_width(img, p1, p4, line_width, color);
        Self::draw_aa_line_with_width(img, p4, tri_left, line_width, color);
        Self::draw_aa_line_with_width(img, tri_left, tip, line_width, color);
        Self::draw_aa_line_with_width(img, tip, tri_right, line_width, color);
        Self::draw_aa_line_with_width(img, tri_right, p3, line_width, color);
        Self::draw_aa_line_with_width(img, p3, p2, line_width, color);
        Self::draw_aa_line_with_width(img, p2, p1, line_width, color);
    }

    /// 渲染实心三角形
    fn render_filled_triangle(
        img: &mut RgbaImage,
        tip: (f32, f32),
        dir: (f32, f32),
        size: f32,
        color: Rgba<u8>,
    ) {
        let perpendicular = (-dir.1, dir.0);
        let back = (tip.0 - dir.0 * size, tip.1 - dir.1 * size);
        let left = (
            back.0 + perpendicular.0 * size * 0.5,
            back.1 + perpendicular.1 * size * 0.5,
        );
        let right = (
            back.0 - perpendicular.0 * size * 0.5,
            back.1 - perpendicular.1 * size * 0.5,
        );

        let points = [
            Self::round_point(tip),
            Self::round_point(left),
            Self::round_point(right),
        ];
        draw_antialiased_polygon_mut(img, &points, color, Self::blend_rgba);
    }

    /// 渲染渐变粗细的线条
    fn render_tapered_line(
        img: &mut RgbaImage,
        start: (f32, f32),
        end: (f32, f32),
        dir: (f32, f32),
        stroke_width: f32,
        head_size: f32,
        color: Rgba<u8>,
    ) {
        let perpendicular = (-dir.1, dir.0);
        let start_width = stroke_width * 0.3;
        let end_width = head_size * 0.5;

        let p1 = (
            start.0 + perpendicular.0 * start_width * 0.5,
            start.1 + perpendicular.1 * start_width * 0.5,
        );
        let p2 = (
            start.0 - perpendicular.0 * start_width * 0.5,
            start.1 - perpendicular.1 * start_width * 0.5,
        );
        let p3 = (
            end.0 - perpendicular.0 * end_width,
            end.1 - perpendicular.1 * end_width,
        );
        let p4 = (
            end.0 + perpendicular.0 * end_width,
            end.1 + perpendicular.1 * end_width,
        );

        let points = [
            Self::round_point(p1),
            Self::round_point(p4),
            Self::round_point(p3),
            Self::round_point(p2),
        ];
        draw_antialiased_polygon_mut(img, &points, color, Self::blend_rgba);
    }

    /// 渲染线段到图像
    fn render_line_to_image(
        &self,
        img: &mut RgbaImage,
        start: (f32, f32),
        end: (f32, f32),
        color: Rgba<u8>,
        stroke_width: f32,
    ) {
        let width = stroke_width.max(1.0);

        match self.line_style {
            LineStyle::Solid => {
                Self::draw_aa_line_with_width(img, start, end, width, color);
            }
            LineStyle::Dashed => {
                Self::render_dashed_line(
                    img,
                    start,
                    end,
                    width,
                    color,
                    stroke_width * 1.5,
                    stroke_width * 0.8,
                );
            }
            LineStyle::Dotted => {
                Self::render_dotted_line(img, start, end, width, color);
            }
            LineStyle::DashDot | LineStyle::DashDotDot => {
                Self::render_dashed_line(
                    img,
                    start,
                    end,
                    width,
                    color,
                    stroke_width * 1.2,
                    stroke_width * 0.6,
                );
            }
        }
    }

    /// 渲染虚线到图像（静态方法）
    fn render_dashed_line(
        img: &mut RgbaImage,
        start: (f32, f32),
        end: (f32, f32),
        width: f32,
        color: Rgba<u8>,
        dash_len: f32,
        gap_len: f32,
    ) {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let total_len = (dx * dx + dy * dy).sqrt();
        if total_len == 0.0 {
            return;
        }

        let dir = (dx / total_len, dy / total_len);
        let mut pos = 0.0;
        let mut is_dash = true;

        while pos < total_len {
            if is_dash {
                let seg_start = (start.0 + dir.0 * pos, start.1 + dir.1 * pos);
                let seg_end_pos = (pos + dash_len).min(total_len);
                let seg_end = (start.0 + dir.0 * seg_end_pos, start.1 + dir.1 * seg_end_pos);
                Self::draw_aa_line_with_width(img, seg_start, seg_end, width, color);
                pos = seg_end_pos;
            } else {
                pos += gap_len;
            }
            is_dash = !is_dash;
        }
    }

    /// 渲染点线到图像（静态方法）
    fn render_dotted_line(
        img: &mut RgbaImage,
        start: (f32, f32),
        end: (f32, f32),
        width: f32,
        color: Rgba<u8>,
    ) {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let total_len = (dx * dx + dy * dy).sqrt();
        if total_len == 0.0 {
            return;
        }

        let dir = (dx / total_len, dy / total_len);
        let dot_gap = (width * 2.0).max(1.0);
        let mut pos = 0.0;

        while pos < total_len {
            let dot_pos = (start.0 + dir.0 * pos, start.1 + dir.1 * pos);
            Self::draw_antialiased_circle(img, dot_pos, width / 2.0, color);
            pos += dot_gap;
        }
    }
}

/// 绘制状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawState {
    #[default]
    Idle,
    Drawing,            // 正在绘制新箭头
    Selected(usize),    // 选中了某个箭头
    MovingStart(usize), // 移动箭头起点
    MovingEnd(usize),   // 移动箭头终点
    MovingWhole(usize), // 移动整个箭头
}

/// 箭头绘制管理器
#[derive(Default)]
pub struct ArrowDrawer {
    pub arrows: Vec<Arrow>,
    pub state: DrawState,
    pub current_start: Option<Pos2>,
    drag_offset: Vec2,
}

impl ArrowDrawer {
    /// 开始绘制新箭头
    pub fn start_drawing(&mut self, pos: Pos2) {
        self.state = DrawState::Drawing;
        self.current_start = Some(pos);
    }

    /// 完成绘制
    pub fn finish_drawing(&mut self, end: Pos2, config: &ArrowConfig) {
        if let Some(start) = self.current_start {
            // 只有拖拽了一定距离才创建箭头
            if (end - start).length() > 10.0 {
                let arrow = Arrow::new(start, end, config);
                self.arrows.push(arrow);
            }
        }
        self.state = DrawState::Idle;
        self.current_start = None;
    }

    /// 取消绘制
    pub fn cancel(&mut self) {
        self.state = DrawState::Idle;
        self.current_start = None;
    }

    /// 尝试选中箭头
    pub fn try_select(&mut self, pos: Pos2) -> bool {
        for (i, arrow) in self.arrows.iter().enumerate().rev() {
            if arrow.hit_test(pos, 5.0) {
                self.state = DrawState::Selected(i);
                return true;
            }
        }
        self.state = DrawState::Idle;
        false
    }

    /// 检测是否点击了选中箭头的端点
    pub fn hit_endpoint(&self, pos: Pos2) -> Option<DrawState> {
        if let DrawState::Selected(idx) = self.state
            && let Some(arrow) = self.arrows.get(idx)
        {
            let tolerance = arrow.stroke_width + 8.0;
            if (pos - arrow.start).length() < tolerance {
                return Some(DrawState::MovingStart(idx));
            }
            if (pos - arrow.end).length() < tolerance {
                return Some(DrawState::MovingEnd(idx));
            }
            if arrow.hit_test(pos, 5.0) {
                return Some(DrawState::MovingWhole(idx));
            }
        }
        None
    }

    /// 开始移动
    pub fn start_move(&mut self, pos: Pos2, new_state: DrawState) {
        if let DrawState::MovingWhole(idx) = new_state
            && let Some(arrow) = self.arrows.get(idx)
        {
            self.drag_offset = pos - arrow.start;
        }
        self.state = new_state;
    }

    /// 更新移动
    pub fn update_move(&mut self, pos: Pos2) {
        match self.state {
            DrawState::MovingStart(idx) => {
                if let Some(arrow) = self.arrows.get_mut(idx) {
                    arrow.start = pos;
                }
            }
            DrawState::MovingEnd(idx) => {
                if let Some(arrow) = self.arrows.get_mut(idx) {
                    arrow.end = pos;
                }
            }
            DrawState::MovingWhole(idx) => {
                if let Some(arrow) = self.arrows.get_mut(idx) {
                    let new_start = pos - self.drag_offset;
                    let delta = new_start - arrow.start;
                    arrow.start = new_start;
                    arrow.end += delta;
                }
            }
            _ => {}
        }
    }

    /// 结束移动
    pub fn finish_move(&mut self) {
        if let DrawState::MovingStart(idx)
        | DrawState::MovingEnd(idx)
        | DrawState::MovingWhole(idx) = self.state
        {
            self.state = DrawState::Selected(idx);
        }
    }

    /// 删除选中的箭头
    pub fn delete_selected(&mut self) {
        if let DrawState::Selected(idx) = self.state {
            if idx < self.arrows.len() {
                self.arrows.remove(idx);
            }
            self.state = DrawState::Idle;
        }
    }

    /// 绘制所有箭头
    pub fn draw_all(&self, painter: &egui::Painter) {
        for arrow in &self.arrows {
            arrow.draw(painter);
        }
    }

    /// 绘制选中状态的端点手柄
    pub fn draw_selection_handles(&self, painter: &egui::Painter) {
        if let DrawState::Selected(idx)
        | DrawState::MovingStart(idx)
        | DrawState::MovingEnd(idx)
        | DrawState::MovingWhole(idx) = self.state
            && let Some(arrow) = self.arrows.get(idx)
        {
            let handle_size = 8.0;
            let handle_color = Color32::from_rgb(0, 122, 255);
            // 起点手柄
            painter.circle_filled(arrow.start, handle_size, Color32::WHITE);
            painter.circle_stroke(arrow.start, handle_size, Stroke::new(2.0, handle_color));
            // 终点手柄
            painter.circle_filled(arrow.end, handle_size, Color32::WHITE);
            painter.circle_stroke(arrow.end, handle_size, Stroke::new(2.0, handle_color));
        }
    }

    /// 绘制正在创建的箭头预览
    pub fn draw_preview(&self, painter: &egui::Painter, current_pos: Pos2, config: &ArrowConfig) {
        if let (DrawState::Drawing, Some(start)) = (self.state, self.current_start) {
            let preview = Arrow::new(start, current_pos, config);
            preview.draw(painter);
        }
    }

    /// 将所有箭头渲染到图像上
    pub fn render_all_to_image(&self, img: &mut RgbaImage, offset_x: f32, offset_y: f32, ppp: f32) {
        for arrow in &self.arrows {
            arrow.render_to_image(img, offset_x, offset_y, ppp);
        }
    }

    /// 检查是否有箭头需要渲染
    pub fn has_arrows(&self) -> bool {
        !self.arrows.is_empty()
    }
}
