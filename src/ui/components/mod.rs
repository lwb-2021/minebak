mod central;
mod window;

use super::{MineBakApp, Signal};

use eframe::egui::{self, CornerRadius, Frame, Margin, panel::Side};

pub(super) fn draw_ui(ctx: &egui::Context, app: &mut MineBakApp) {
    const PADDING: i8 = 12;
    let frame_central = egui::containers::Frame {
        inner_margin: Margin::same(PADDING),
        ..Frame::central_panel(&ctx.style())
    };
    let frame_side = egui::containers::Frame {
        inner_margin: Margin::same(PADDING),
        ..Frame::side_top_panel(&ctx.style())
    };

    let frame_window = egui::containers::Frame {
        inner_margin: Margin::same(PADDING),
        corner_radius: CornerRadius::same(8),
        ..Frame::window(&ctx.style())
    };
    window::show_windows(ctx, app, frame_window);
    action_buttons(ctx, app, frame_side);
    central::central(ctx, app, frame_central);
}

fn action_buttons(ctx: &egui::Context, app: &mut MineBakApp, frame: egui::containers::Frame) {
    egui::SidePanel::new(Side::Right, "action_buttons")
        .frame(frame)
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                if ui.button("重新扫描").clicked() {
                    app.sender.send(Signal::Rescan).unwrap();
                }
                if ui.button("运行备份").clicked() {
                    app.sender.send(Signal::RunBackup).unwrap();
                }
                if ui.button("添加存档").clicked() {
                    app.states.window_add_save_show = true;
                }
                if ui.button("设置").clicked() {
                    app.states.window_settings_show = true;
                }
            });
        });
}
