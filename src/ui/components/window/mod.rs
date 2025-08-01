mod add_save_window;
mod recover_window;

use crate::ui::MineBakApp;


use eframe::egui;



pub(super) fn show_windows(
    ctx: &egui::Context,
    app: &mut MineBakApp,
    frame: egui::containers::Frame,
) {
    add_save_window::show(ctx, app, frame.clone());
    recover_window::show(ctx, app, frame);
}



