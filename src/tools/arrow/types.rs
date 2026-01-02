use eframe::egui::Color32;

/// 箭头类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowType {
    #[default]
    Single, // 直线箭头 (单向)
    Double, // 双向箭头
    Hollow, // 空心箭头
    Filled, // 实心箭头
}

impl ArrowType {
    pub fn all() -> &'static [ArrowType] {
        &[
            ArrowType::Single,
            ArrowType::Double,
            ArrowType::Hollow,
            ArrowType::Filled,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ArrowType::Single => "Single",
            ArrowType::Double => "Double",
            ArrowType::Hollow => "Hollow",
            ArrowType::Filled => "Filled",
        }
    }
}

/// 线段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineStyle {
    #[default]
    Solid, // 实线
    Dashed,     // 虚线
    Dotted,     // 点线
    DashDot,    // 点划线
    DashDotDot, // 双点划线
}

impl LineStyle {
    pub fn all() -> &'static [LineStyle] {
        &[
            LineStyle::Solid,
            LineStyle::Dashed,
            LineStyle::Dotted,
            LineStyle::DashDot,
            LineStyle::DashDotDot,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            LineStyle::Solid => "Solid",
            LineStyle::Dashed => "Dashed",
            LineStyle::Dotted => "Dotted",
            LineStyle::DashDot => "Dash-Dot",
            LineStyle::DashDotDot => "Dash-Dot-Dot",
        }
    }
}

/// 预设颜色
pub const PRESET_COLORS: &[(Color32, &str)] = &[
    (Color32::RED, "Red"),
    (Color32::YELLOW, "Yellow"),
    (Color32::GREEN, "Green"),
    (Color32::BLUE, "Blue"),
    (Color32::BLACK, "Black"),
    (Color32::WHITE, "White"),
];

/// 预设大小
pub const PRESET_SIZES: &[f32] = &[12.0, 6.0];
