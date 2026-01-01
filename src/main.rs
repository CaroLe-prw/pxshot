use eframe::egui;
use pxshot::App;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("pxshot")
            .with_inner_size([520.0, 320.0])
            // overlay 要看到桌面，需要透明窗口
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native(
        "pxshot",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}
