use eframe::egui::{self, Color32, Pos2, Rect, Sense, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    None,
    Cancel,
    Confirm,
    Arrow,
}

pub struct Toolbar {
    pub btn_size: Vec2,
    pub spacing: f32,
}

impl Default for Toolbar {
    fn default() -> Self {
        Self {
            btn_size: Vec2::new(32.0, 32.0),
            spacing: 8.0,
        }
    }
}

impl Toolbar {
    pub fn calc_rect(&self, selection: Rect, screen: Rect) -> Rect {
        let btn_count = 3.0;
        let width = self.btn_size.x * btn_count + self.spacing * (btn_count - 1.0) + 16.0;
        let height = self.btn_size.y + 12.0;

        // 默认放选区下方
        let mut center = Pos2::new(selection.center().x, selection.max.y + 8.0 + height / 2.0);

        // 如果超出底部，放上方
        if center.y + height / 2.0 > screen.max.y - 8.0 {
            center.y = selection.min.y - 8.0 - height / 2.0;
        }

        // clamp 到屏幕内
        center.x = center.x.clamp(
            screen.min.x + width / 2.0 + 8.0,
            screen.max.x - width / 2.0 - 8.0,
        );
        center.y = center.y.clamp(
            screen.min.y + height / 2.0 + 8.0,
            screen.max.y - height / 2.0 - 8.0,
        );

        Rect::from_center_size(center, Vec2::new(width, height))
    }

    /// 绘制工具栏并返回点击的动作
    pub fn show(&self, ui: &mut egui::Ui, toolbar_rect: Rect) -> ToolbarAction {
        let painter = ui.painter();

        // 背景
        painter.rect_filled(toolbar_rect, 6.0, Color32::from_rgb(240, 240, 240));

        let cx = toolbar_rect.center().x;
        let cy = toolbar_rect.center().y;

        let mut action = ToolbarAction::None;

        // Cancel 按钮
        let cancel_rect = Rect::from_center_size(
            Pos2::new(cx - self.btn_size.x - self.spacing, cy),
            self.btn_size,
        );
        let cancel_img = egui::Image::new(egui::include_image!("../../assets/icons/close.png"))
            .fit_to_exact_size(self.btn_size);
        if ui
            .put(cancel_rect, cancel_img.sense(Sense::click()))
            .clicked()
        {
            action = ToolbarAction::Cancel;
        }

        // Confirm 按钮
        let confirm_rect = Rect::from_center_size(Pos2::new(cx, cy), self.btn_size);
        let check_img = egui::Image::new(egui::include_image!("../../assets/icons/check.png"))
            .fit_to_exact_size(self.btn_size);
        if ui
            .put(confirm_rect, check_img.sense(Sense::click()))
            .clicked()
        {
            action = ToolbarAction::Confirm;
        }

        // Arrow 按钮
        let arrow_rect = Rect::from_center_size(
            Pos2::new(cx + self.btn_size.x + self.spacing, cy),
            self.btn_size,
        );
        let arrow_img = egui::Image::new(egui::include_image!("../../assets/icons/arrow.png"))
            .fit_to_exact_size(self.btn_size);
        if ui
            .put(arrow_rect, arrow_img.sense(Sense::click()))
            .clicked()
        {
            action = ToolbarAction::Arrow;
        }

        action
    }
}
