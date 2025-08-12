use std::env;

use crate::ui::MineBakApp;

use eframe::egui::{self, ImageSource, RichText, Ui};

pub(super) fn central(ctx: &egui::Context, app: &mut MineBakApp, frame: egui::containers::Frame) {
    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
        instances_list(ui, app);
    });
}

fn instances_list(ui: &mut Ui, app: &mut MineBakApp) {
    ui.hyperlink_to(
        "点击此链接汇报bug",
        "https://github.com/lwb-2021/minebak/issues",
    );
    ui.hyperlink_to(
        "记得提交这个文件",
        format!("file://{}/minebak.log", env::temp_dir().to_str().unwrap()),
    );
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
