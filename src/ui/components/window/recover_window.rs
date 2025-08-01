use crate::ui::{Signal, MineBakApp};


use chrono::DateTime;
use eframe::egui;

pub(super) fn show(ctx: &egui::Context, app: &mut MineBakApp, frame: egui::containers::Frame) {
    egui::Window::new("恢复")
        .frame(frame)
        .open(&mut app.states.window_recover_show)
        .show(ctx, |ui| {
            if app.states.recover_current_save.is_none() {
                ui.label("出Bug了，请重新打开");
                return;
            }
            if !app.states.window_recover_refreshed {
                let mut wait = ui.label("列出备份中，请等待");
                let result = app
                    .states
                    .recover_current_save
                    .clone()
                    .unwrap()
                    .list_backups(app.config.read().unwrap().backup_root.clone());
                if result.is_err() {
                    log::error!("{}", result.as_ref().unwrap_err());
                    app.states.err_list.push(result.unwrap_err());
                    return;
                }
                app.states.window_recover_refreshed = true;
                wait.set_close();
                let result = result.unwrap();
                app.states.recover_backup_names = result;
            }

            ui.heading(
                "恢复 ".to_string() + &app.states.recover_current_save.as_ref().unwrap().name,
            );
            ui.label("请关闭相应的Minecraft存档后进行恢复！");
            for item in app.states.recover_backup_names.iter() {
                let date = DateTime::from_timestamp_millis(item.parse().unwrap()).unwrap();
                ui.horizontal(|ui| {
                    ui.label(date.format("%Y-%m-%d %H:%M").to_string());
                    if ui.button("恢复").clicked() {
                        app.sender
                            .send(Signal::Recover {
                                save: app.states.recover_current_save.clone().unwrap(),
                                timestamp: date.timestamp_millis().to_string(),
                            })
                            .unwrap();
                    }
                });
            }
        });
}
