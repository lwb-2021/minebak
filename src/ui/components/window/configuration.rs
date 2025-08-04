use crate::ui::MineBakApp;

use eframe::egui;

pub(super) fn show(ctx: &egui::Context, app: &mut MineBakApp, frame: egui::containers::Frame) {
    egui::Window::new("恢复")
        .frame(frame)
        .open(&mut app.states.window_settings_show)
        .show(ctx, |ui| {
            
        });
}
