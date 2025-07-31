use crate::{config::Config};

use std::sync::{mpsc::{Receiver, Sender}, RwLock};

use eframe::egui::{self, panel::Side, Margin, RichText};


pub(super) fn draw_ui(ctx: &egui::Context, config: &RwLock<Config>, sender: &Sender<u8>, err: &Receiver<anyhow::Error>) { 
    let layout = egui::containers::Frame {
        inner_margin: Margin::same(12),
        ..Default::default()
    };

    central(ctx, config, sender, layout.clone());
    action_buttons(ctx, config, sender, layout);

}

fn central(ctx: &egui::Context, config: &RwLock<Config>, sender: &Sender<u8>, layout: egui::containers::Frame) {
    egui::CentralPanel::default().frame(layout).show(ctx, |ui|{
        for instance_root in (*config.read().unwrap()).instance_roots.iter() {
            ui.collapsing(instance_root.name.clone(), |ui| {
                for instance in instance_root.instances.iter() {
                    ui.collapsing(instance.name.clone(), |ui| {
                        for save in instance.saves.iter() {
                            ui.heading(RichText::new(save.name.clone()));
                        }
                    });
                }   
            });
        }
    });
}

fn action_buttons(ctx: &egui::Context, config: &RwLock<Config>, sender: &Sender<u8>, layout: egui::containers::Frame) {
    egui::SidePanel::new(Side::Right, "action_buttons").frame(layout).show(ctx, |ui| {
        if ui.button("重新扫描").clicked() {
            sender.send(0).unwrap();
        } 
        if ui.button("运行备份").clicked() {
            sender.send(1).unwrap();
        }
        if ui.button("添加存档").clicked() {
            sender.send(2).unwrap();
        }
    });
}
