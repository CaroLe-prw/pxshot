use eframe::egui::{self, CursorIcon, Pos2, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitZone {
    None,
    Inside,      // 中间 - 移动
    TopLeft,     // 左上角
    TopRight,    // 右上角
    BottomLeft,  // 左下角
    BottomRight, // 右下角
    Top,         // 上边
    Bottom,      // 下边
    Left,        // 左边
    Right,       // 右边
}

impl HitZone {
    const EDGE_SIZE: f32 = 8.0;

    ///检测鼠标在选区的哪个区域
    pub fn detect(pos: Pos2, rect: Rect) -> Self {
        let expanded = rect.expand(Self::EDGE_SIZE);
        if !expanded.contains(pos) {
            return HitZone::None;
        }

        let near_left = (pos.x - rect.min.x).abs() < Self::EDGE_SIZE;
        let near_right = (pos.x - rect.max.x).abs() < Self::EDGE_SIZE;
        let near_top = (pos.y - rect.min.y).abs() < Self::EDGE_SIZE;
        let near_bottom = (pos.y - rect.max.y).abs() < Self::EDGE_SIZE;

        match (near_left, near_right, near_top, near_bottom) {
            (true, _, true, _) => HitZone::TopLeft,
            (_, true, true, _) => HitZone::TopRight,
            (true, _, _, true) => HitZone::BottomLeft,
            (_, true, _, true) => HitZone::BottomRight,
            (true, _, _, _) => HitZone::Left,
            (_, true, _, _) => HitZone::Right,
            (_, _, true, _) => HitZone::Top,
            (_, _, _, true) => HitZone::Bottom,
            _ if rect.contains(pos) => HitZone::Inside,
            _ => HitZone::None,
        }
    }

    ///对应的光标样式
    pub fn cursor(&self) -> CursorIcon {
        match self {
            HitZone::None => CursorIcon::Default,
            HitZone::Inside => CursorIcon::Move,
            HitZone::TopLeft | HitZone::BottomRight => CursorIcon::ResizeNwSe,
            HitZone::TopRight | HitZone::BottomLeft => CursorIcon::ResizeNeSw,
            HitZone::Top | HitZone::Bottom => CursorIcon::ResizeVertical,
            HitZone::Left | HitZone::Right => CursorIcon::ResizeHorizontal,
        }
    }

    /// 根据拖拽增量调整选区
    pub fn apply_drag(&self, rect: Rect, delta: egui::Vec2, screen: Rect) -> Rect {
        let mut min = rect.min;
        let mut max = rect.max;

        match self {
            HitZone::Inside => {
                min += delta;
                max += delta;
            }
            HitZone::TopLeft => {
                min += delta;
            }
            HitZone::TopRight => {
                min.y += delta.y;
                max.x += delta.x;
            }
            HitZone::BottomLeft => {
                min.x += delta.x;
                max.y += delta.y;
            }
            HitZone::BottomRight => {
                max += delta;
            }
            HitZone::Top => {
                min.y += delta.y;
            }
            HitZone::Bottom => {
                max.y += delta.y;
            }
            HitZone::Left => {
                min.x += delta.x;
            }
            HitZone::Right => {
                max.x += delta.x;
            }
            HitZone::None => {}
        }

        // 确保 min < max
        if min.x > max.x {
            std::mem::swap(&mut min.x, &mut max.x);
        }
        if min.y > max.y {
            std::mem::swap(&mut min.y, &mut max.y);
        }

        // 限制在屏幕内
        Rect::from_min_max(min, max).intersect(screen)
    }
}
