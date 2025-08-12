use crate::{
    cloud_sync::CloudService,
    ui::{AppSettings, MineBakApp},
};

use eframe::egui::{self, RichText, Window};
use rfd::FileDialog;

pub(super) fn show(ctx: &egui::Context, app: &mut MineBakApp, frame: egui::containers::Frame) {
    egui::Window::new("设置")
        .frame(frame)
        .open(&mut app.states.window_settings_show)
        .show(ctx, |ui| {
            ui.collapsing("基础", |ui| {
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
                        if let Some(folder) =
                            FileDialog::new().set_title("选择备份文件夹").pick_folder()
                        {
                            app.states.settings.backup_root = folder.to_string_lossy().to_string();
                        }
                    }
                });
            });
            ui.collapsing("云同步", |ui| {
                ui.horizontal(|ui| {
                    ui.label("警告：云同步可能有成吨的bug");
                    if ui.button("添加WebDAV").clicked() {
                        app.states.webdav_window_open = true;
                    }
                    if ui.button("添加RClone同步").clicked() {
                        app.states.rclone_window_open = true;
                    }
                });

                let config = app.config.read().unwrap();
                let services: Vec<_> = config.cloud_services.keys().cloned().collect();
                drop(config);
                ui.collapsing("管理云服务", |ui| {
                    for key in services {
                        ui.horizontal(|ui| {
                            ui.label(key.clone());
                            if ui.button("删除").clicked() {
                                app.config.write().unwrap().cloud_services.remove(&key);
                            }
                        });
                    }
                });
            });
            ui.horizontal(|ui| {
                if ui.button("应用").clicked() {
                    app.states.settings.save(&mut app.config.write().unwrap());
                }
                if ui.button("撤销").clicked() {
                    app.states.settings = AppSettings::from(&app.config.read().unwrap())
                }
            });

            Window::new("添加rclone同步")
                .open(&mut app.states.webdav_window_open)
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Remote");
                            ui.text_edit_singleline(&mut app.states.webdav_endpoint);
                        });
                        if ui.button("应用").clicked() {
                            app.config.write().unwrap().cloud_services.insert(
                                app.states.rclone_remote.clone()
                                    + "@RClone",
                                CloudService::RClone {
                                    remote: app.states.rclone_remote.clone()
                                },
                            );
                        }
                    });
                });

            Window::new("添加WebDAV同步")
                .open(&mut app.states.webdav_window_open)
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("服务器");
                            ui.text_edit_singleline(&mut app.states.webdav_endpoint);
                        });
                        ui.horizontal(|ui| {
                            ui.label("用户名");
                            ui.text_edit_singleline(&mut app.states.webdav_username);
                        });
                        ui.horizontal(|ui| {
                            ui.label("密码");
                            egui::TextEdit::singleline(&mut app.states.webdav_password)
                                .password(true)
                                .show(ui);
                        });

                        ui.label(
                            RichText::new(&app.states.webdav_window_err)
                                .color(ui.ctx().style().visuals.error_fg_color),
                        );
                        if ui.button("应用").clicked() {
                            app.config.write().unwrap().cloud_services.insert(
                                app.states.webdav_username.clone()
                                    + "@"
                                    + &app.states.webdav_endpoint,
                                CloudService::WebDAV {
                                    endpoint: app.states.webdav_endpoint.clone(),
                                    username: app.states.webdav_username.clone(),
                                    password: app.states.webdav_password.clone(),
                                    init: false,
                                    client: None,
                                },
                            );
                        }
                    });
                });
        });
}
