mod model;
mod parser;
mod writer;
mod app;

use app::FocusFlowApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("FocusFlow — HOI4 Focus Tree Editor"),
        ..Default::default()
    };
    
    eframe::run_native(
        "FocusFlow",
        native_options,
        Box::new(|cc| Ok(Box::new(FocusFlowApp::new(cc)))),
    )
}
