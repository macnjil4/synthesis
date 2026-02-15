mod app;
pub mod oscilloscope;
mod tenori_app;

pub fn run() {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1400.0, 850.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Synthwave",
        options,
        Box::new(|cc| Ok(Box::new(app::SynthApp::new(cc)))),
    )
    .expect("failed to run eframe");
}

pub fn run_tenori() {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1100.0, 750.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Tenori-Synth",
        options,
        Box::new(|cc| Ok(Box::new(tenori_app::TenoriApp::new(cc)))),
    )
    .expect("failed to run eframe");
}
