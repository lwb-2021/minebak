use crate::ui::{AppSettings, MineBakApp};

use eframe::egui;
use rfd::FileDialog;

pub(super) fn show(ctx: &egui::Context, app: &mut MineBakApp, frame: egui::containers::Frame) {
    egui::Window::new("设置")
        .frame(frame)
        .open(&mut app.states.window_settings_show)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("备份间隔(分钟)");
                ui.add(egui::Slider::new(
                    &mut app.states.settings.backup_duration_mins,
                    10..=360,
                ));
            });

            ui.horizontal(|ui| {
                ui.label("备份文件夹");
                ui.text_edit_singleline(&mut app.states.settings.backup_root);
                if ui.button("浏览").clicked() {
                    if let Some(folder) = FileDialog::new().set_title("选择备份文件夹").pick_folder() {
                        app.states.settings.backup_root = folder.to_string_lossy().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                // if ui.button("确定").clicked() {
                    
                // }
                
                if ui.button("应用").clicked() {
                    app.states.settings.save(&mut app.config.write().unwrap());
                }
                if ui.button("撤销").clicked() {
                    app.states.settings = AppSettings::from(app.config.read().unwrap().clone())
                }
            })
        });
}
