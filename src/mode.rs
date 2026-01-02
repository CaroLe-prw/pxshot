use std::time::Instant;

use eframe::egui::{Pos2, Rect};

use crate::{capture::RectPx, overlay::HitZone};

#[derive(Clone, Copy, Debug, Default)]
pub enum Mode {
    #[default]
    Idle,
    Selecting {
        start: Pos2,
        end: Pos2,
    },
    Selected {
        rect: Rect,
        dragging: Option<HitZone>,
    },
    PendingCapture {
        rect_px: RectPx,
        rect_points: Rect, // 选区的逻辑坐标（用于箭头渲染）
        hidden_at: Instant,
    },
}
