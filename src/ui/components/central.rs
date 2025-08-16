use std::{env, iter::zip};

use crate::ui::MineBakApp;

use eframe::egui::{self, CollapsingHeader, Image, RichText, Ui};

#[inline]
pub(super) fn central(ctx: &egui::Context, app: &mut MineBakApp, frame: egui::containers::Frame) {
    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
        content(ui, app);
    });
}
#[inline]
fn content(ui: &mut Ui, app: &mut MineBakApp) {
    ui.hyperlink_to(
        "点击此链接汇报bug",
        "https://github.com/lwb-2021/minebak/issues",
    );
    ui.hyperlink_to(
        "记得提交这个文件",
        format!("file://{}/minebak.log", env::temp_dir().to_str().unwrap()),
    );
    CollapsingHeader::new("存档列表")
        .default_open(true)
        .show(ui, |ui| {
            save_list(ui, app);
        });
}

#[inline]
fn save_list(ui: &mut Ui, app: &mut MineBakApp) {
    let instance_roots = &app.config.read().unwrap().instance_roots;
    for (instance_root, i1) in zip(instance_roots.iter(), 0..instance_roots.len()) {
        for (instance, i2) in zip(instance_root.instances.iter(), 0..instance_root.instances.len()) {
            for (save, i3) in zip(instance.saves.iter(), 0..instance.saves.len()) {
                egui::Frame::group(&ui.ctx().style())
                    .fill(ui.ctx().style().visuals.faint_bg_color)
                    .corner_radius(4)
                    .outer_margin(4)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if save.image.is_some() {
                                ui.add(
                                    Image::new(
                                        "file://".to_string()
                                            + &save
                                                .image
                                                .as_ref()
                                                .unwrap()
                                                .to_string_lossy()
                                                .to_string(),
                                    )
                                    .fit_to_exact_size((64.0, 64.0).into())
                                    .corner_radius(4),
                                );
                            } else {
                                ui.add(
                                    Image::new("file://")
                                        .fit_to_exact_size((64.0, 64.0).into())
                                        .corner_radius(4),
                                );
                            }
                            ui.vertical_centered_justified(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(save.name.clone()).size(16.0));
                                    ui.weak(format!("{}/{}", instance_root.name, instance.name));
                                });
                                ui.horizontal(|ui| {
                                    ui.label(&save.description)
                                });
                                ui.columns(3, |ui| {
                                    ui[0].vertical_centered_justified(|ui| {
                                        if ui.button("编辑（TODO）").clicked() {
                                            app.states.edit_save_index = [i1, i2, i3];  
                                            app.states.edit_save_info = save.clone();
                                            app.states.save_edit_window_open = true;
                                        }
                                    });
                                    ui[1].vertical_centered_justified(|ui| {
                                        if ui.button("备份（TODO）").clicked() {}
                                    });
                                    ui[2].vertical_centered_justified(|ui| {
                                        if ui.button("恢复").clicked() {
                                            app.states.window_recover_refreshed = false;
                                            app.states.recover_current_save = Some(save.clone());
                                            app.states.window_recover_show = true;
                                        }
                                    });
                                });
                            });
                        });
                    });
            }
        }
    }
}
