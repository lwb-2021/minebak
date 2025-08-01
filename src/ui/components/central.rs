use crate::ui::MineBakApp;

use eframe::egui::{self, ImageSource, RichText, Ui};

pub(super) fn central(ctx: &egui::Context, app: &mut MineBakApp, frame: egui::containers::Frame) {
    egui::CentralPanel::default()
        .frame(frame)
        .show(ctx, |ui| {
            instances_list(ui, app);
        });
}

fn instances_list(ui: &mut Ui, app: &mut MineBakApp) {
    for instance_root in (*app.config.read().unwrap()).instance_roots.iter() {
        ui.collapsing(instance_root.name.clone(), |ui| {
            for instance in instance_root.instances.iter() {
                ui.collapsing(instance.name.clone(), |ui| {
                    for save in instance.saves.iter() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                if save.image.is_some() {
                                    ui.image(ImageSource::Uri(
                                        ("file://".to_string()
                                            + &save
                                                .image
                                                .as_ref()
                                                .unwrap()
                                                .to_string_lossy()
                                                .to_string())
                                            .into(),
                                    ));
                                }
                                ui.heading(RichText::new(save.name.clone()));
                                if ui.button("恢复").clicked() {
                                    app.states.window_recover_refreshed = false;
                                    app.states.recover_current_save = Some(save.clone());
                                    app.states.window_recover_show = true;
                                }
                            })
                        });
                    }
                });
            }
        });
    }
}

