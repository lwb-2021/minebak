use eframe::egui::{self, Window};

use crate::ui::MineBakApp;

pub(super) fn show(ctx: &egui::Context, app: &mut MineBakApp, frame: egui::containers::Frame) {
    Window::new("编辑存档")
        .frame(frame)
        .open(&mut app.states.save_edit_window_open)
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.label("存档名称");
                    ui.text_edit_singleline(&mut app.states.edit_save_info.name);
                });

                ui.horizontal(|ui| {
                    ui.label("存档描述");
                    ui.text_edit_singleline(&mut app.states.edit_save_info.description);
                });

                if ui.button("应用").clicked() {
                    let [i1, i2, i3] = app.states.edit_save_index;
                    let source =
                        &mut app.config.write().unwrap().instance_roots[i1].instances[i2].saves[i3];
                    source.name = app.states.edit_save_info.name.clone();
                    source.description = app.states.edit_save_info.description.clone();
                    app.sender.send(crate::ui::Signal::SaveConfig).unwrap();
                }
            });
        });
}
