use eframe::egui::Color32;

use super::types::{ArrowType, LineStyle};

/// 箭头工具配置
#[derive(Debug, Clone)]
pub struct ArrowConfig {
    pub arrow_type: ArrowType,
    pub line_style: LineStyle,
    pub color: Color32,
    pub stroke_width: f32,
}

impl Default for ArrowConfig {
    fn default() -> Self {
        Self {
            arrow_type: ArrowType::Single,
            line_style: LineStyle::Solid,
            color: Color32::RED,
            stroke_width: 4.0,
        }
    }
}
