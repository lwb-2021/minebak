use crate::ui::{Signal, MineBakApp};

use std::{env, path::PathBuf};

use rfd::FileDialog;
use eframe::egui::{self, RichText};

pub(super) fn show(ctx: &egui::Context, app: &mut MineBakApp, frame: egui::containers::Frame) {
    egui::Window::new("添加存档")
        .frame(frame)
        .open(&mut app.states.window_add_save_show)
        .show(ctx, |window| {
            window.horizontal(|ui| {
                ui.label("实例文件夹")
                    .on_hover_text_at_pointer("实例的.minecraft文件夹或PrismLauncher的实例文件夹");
                ui.text_edit_singleline(&mut app.states.add_save_window_path_input);
                if ui.button("浏览").clicked() {
                    if let Some(path) = FileDialog::new()
                        .set_title("请选择实例的.minecraft文件夹或PrismLauncher的实例文件夹")
                        .pick_folder()
                    {
                        app.states.add_save_window_path_input = path.to_string_lossy().to_string();
                    }
                }
            });
            window.horizontal(|ui| {
                if ui.button("自动识别官方启动器").clicked() {
                    if let Some(mut home) = env::home_dir() {
                        home.push(".minecraft");
                        if !home.exists() {
                            app.states.add_save_window_error_message =
                                "识别失败：无法找到官方启动器目录".to_string();
                        } else {
                            app.states.add_save_window_path_input =
                                home.to_string_lossy().to_string();
                        }
                    } else {
                        app.states.add_save_window_error_message =
                            "识别失败：无法获取家目录".to_string();
                    }
                }
                if ui.button("自动识别Prism Launcher").clicked() {
                    if let Some(mut home) = env::home_dir() {
                        let mut failed = false;
                        #[cfg(target_os = "linux")]
                        {
                            home.push(".local/share/PrismLauncher");
                            if !home.exists() {
                                // For Flatpak
                                home = env::home_dir().unwrap();
                                home.push(
                                    ".var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher",
                                );
                                if !home.exists() {
                                    app.states.add_save_window_error_message =
                                        "识别失败：无法找到Prism Launcher".to_string();
                                    failed = true;
                                }
                                home.clear();
                            }
                        }
                        #[cfg(target_os = "windows")]
                        {
                            app.states.add_save_window_error_message =
                                "识别失败：Windows未实现".to_string();
                            failed = true;
                        }

                        if !failed {
                            home.push("instances");
                            app.states.add_save_window_path_input =
                                home.to_string_lossy().to_string();
                        }
                    } else {
                        app.states.add_save_window_error_message =
                            "识别失败：无法获取家目录".to_string();
                    }
                }
            });
            window.separator();
            window.horizontal(|ui| {
                ui.label("名称");
                ui.text_edit_singleline(&mut app.states.add_save_window_name_input);
                if ui.button("自动识别").clicked() {
                    let path = &app.states.add_save_window_path_input;
                    let mut name = String::new();
                    if path.contains("PrismLauncher") {
                        name = "Prism Launcher".to_string();
                    } else if path.contains(".minecraft") {
                        let mut path = PathBuf::from(path);
                        while path.file_name().unwrap().to_string_lossy().to_string()
                            != ".minecraft"
                        {
                            path.pop();
                        }
                        path.pop();
                        if env::home_dir().is_some() && path == env::home_dir().unwrap() {
                            name = "Official Launcher".to_string();
                        } else {
                            let pth = path
                                .parent()
                                .map(|p| p.to_path_buf())
                                .unwrap_or_else(PathBuf::new);
                            name = pth.to_string_lossy().to_string();
                        }
                    }
                    if !name.is_empty() {
                        app.states.add_save_window_name_input = name;
                    } else {
                        app.states.add_save_window_error_message =
                            "自动识别失败，请手动输入".to_string();
                    }
                }
            });
            window.separator();
            window.label(
                RichText::new(&app.states.add_save_window_error_message)
                    .color(window.style().visuals.error_fg_color),
            );
            window.horizontal(|ui| {
                if ui.button("添加MultiMC实例").clicked() {
                    let path = PathBuf::from(app.states.add_save_window_path_input.clone());
                    if !path.exists() || !path.is_dir() {
                        app.states.add_save_window_error_message =
                            "路径不存在或不是一个文件夹".to_string();
                    } else {
                        app.sender
                            .send(Signal::AddInstance {
                                name: app.states.add_save_window_name_input.to_string(),
                                path: path,
                                multimc: true,
                                version_isolated: true,
                            })
                            .unwrap();
                    }
                }
                if ui.button("添加版本隔离实例").clicked() {
                    todo!("添加版本隔离实例")
                }
                if ui.button("添加普通实例").clicked() {
                    todo!("添加普通实例")
                }
            });
        });
}