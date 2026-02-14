use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use fundsp::snoop::Snoop;

const NUM_POINTS: usize = 512;

/// Draw an oscilloscope widget showing the current audio waveform.
pub fn draw(ui: &mut egui::Ui, snoop_left: &mut Option<Snoop>, snoop_right: &mut Option<Snoop>) {
    if let Some(s) = snoop_left.as_mut() {
        s.update();
    }
    if let Some(s) = snoop_right.as_mut() {
        s.update();
    }

    let left_points: PlotPoints = if let Some(snoop) = snoop_left.as_ref() {
        PlotPoints::new(
            (0..NUM_POINTS)
                .map(|i| [(NUM_POINTS - 1 - i) as f64, snoop.at(i) as f64])
                .collect(),
        )
    } else {
        PlotPoints::new((0..NUM_POINTS).map(|i| [i as f64, 0.0]).collect())
    };

    let right_points: PlotPoints = if let Some(snoop) = snoop_right.as_ref() {
        PlotPoints::new(
            (0..NUM_POINTS)
                .map(|i| [(NUM_POINTS - 1 - i) as f64, snoop.at(i) as f64])
                .collect(),
        )
    } else {
        PlotPoints::new((0..NUM_POINTS).map(|i| [i as f64, 0.0]).collect())
    };

    let left_line = Line::new("Left", left_points);
    let right_line = Line::new("Right", right_points);

    Plot::new("oscilloscope")
        .height(200.0)
        .include_y(-1.0)
        .include_y(1.0)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .show(ui, |plot_ui| {
            plot_ui.line(left_line);
            plot_ui.line(right_line);
        });
}
