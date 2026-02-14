mod app;
mod keyboard;
mod oscilloscope;

pub fn run() {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([700.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Synthesis",
        options,
        Box::new(|cc| Ok(Box::new(app::SynthApp::new(cc)))),
    )
    .expect("failed to run eframe");
}
