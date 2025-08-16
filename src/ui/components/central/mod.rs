mod save_list;


use crate::ui::MineBakApp;

use eframe::egui;

#[inline]
pub(super) fn central(ctx: &egui::Context, app: &mut MineBakApp, frame: egui::containers::Frame) {
    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
        match app.states.central_panel {
            0 => {
                save_list::content(ui, app);
            },
            _ => unreachable!()
        }
        
    });
}
