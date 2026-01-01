use eframe::egui;
use egui::Pos2;
use egui::viewport::{ViewportCommand, WindowLevel};
use std::time::Duration;

use crate::capture::{RgbaImage, capture_region};
use crate::mode::Mode;

#[derive(Default)]
pub struct App {
    pub(crate) mode: Mode,
    pub(crate) screenshot: Option<RgbaImage>,
    pub(crate) texture: Option<egui::TextureHandle>,
    image_loaders_installed: bool,
}

fn image_to_texture(ctx: &egui::Context, img: &RgbaImage) -> egui::TextureHandle {
    let size = [img.width() as usize, img.height() as usize];
    let pixels = img.clone().into_raw();
    let color = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
    ctx.load_texture("shot", color, Default::default())
}

impl App {
    const CAPTURE_DELAY_MS: u64 = 180;

    pub fn enter_overlay(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(true));
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        ctx.request_repaint();
    }

    pub fn exit_overlay(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
        ctx.request_repaint();
    }

    pub fn cancel_overlay(&mut self, ctx: &egui::Context) {
        self.mode = Mode::Idle;
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        self.exit_overlay(ctx);
    }

    fn idle_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("pxshot");

            if ui.button("Region screenshot").clicked() {
                self.screenshot = None;
                self.texture = None;
                self.enter_overlay(ctx);
                self.mode = Mode::Selecting {
                    start: Pos2::new(0.0, 0.0),
                    end: Pos2::new(0.0, 0.0),
                };
            }

            if let Some(img) = &self.screenshot
                && self.texture.is_none()
            {
                self.texture = Some(image_to_texture(ctx, img));
            }

            if let Some(tex) = &self.texture {
                ui.separator();
                ui.label(format!("Captured: {}x{}", tex.size()[0], tex.size()[1]));
                let ppp = ctx.pixels_per_point();
                let display_size = tex.size_vec2() / ppp;
                ui.image((tex.id(), display_size));

                if ui.button("Save screenshot.png").clicked()
                    && let Some(img) = &self.screenshot
                {
                    let _ = img.save("screenshot.png");
                }
            }
        });
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // overlay “洞”里要跟桌面一样亮，这里必须全透明
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // loaders 只装一次
        if !self.image_loaders_installed {
            egui_extras::install_image_loaders(ctx);
            self.image_loaders_installed = true;
        }

        // PendingCapture：这一段期间不画任何东西（全透明窗口），等合成器刷新后截图
        if let Mode::PendingCapture { rect_px, hidden_at } = self.mode {
            // 驱动帧循环
            ctx.request_repaint_after(Duration::from_millis(16));

            if hidden_at.elapsed() < Duration::from_millis(Self::CAPTURE_DELAY_MS) {
                return;
            }

            match capture_region(rect_px) {
                Ok(img) => {
                    self.screenshot = Some(img);
                    self.texture = None;
                }
                Err(e) => eprintln!("capture failed: {e:?}"),
            }

            self.mode = Mode::Idle;
            self.exit_overlay(ctx);
            return;
        }

        match self.mode {
            Mode::Idle => self.idle_ui(ctx),
            Mode::Selecting { .. } => self.overlay_selecting_ui(ctx),
            Mode::Selected { .. } => self.overlay_selected_ui(ctx),
            Mode::PendingCapture { .. } => {}
        }
    }
}
