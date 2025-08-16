use std::sync::Arc;

use eframe::{
    CreationContext,
    egui::{self, Context, FontData, FontTweak},
};

use super::States;

pub(super) fn set_font(cc: &CreationContext<'_>) {
    let mut fonts = egui::FontDefinitions::default();

    let font_tweak = FontTweak {
        ..Default::default()
    };

    fonts.font_data.insert(
        "Source Han Sans SC".to_string(),
        Arc::new(
            FontData::from_static(include_bytes!("../../resources/SourceHanSansCN-VF.otf"))
                .tweak(font_tweak),
        ),
    );

    let proportional = fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap();
    proportional.insert(0, "Source Han Sans SC".to_string());
    cc.egui_ctx.set_fonts(fonts);
}


pub(super) fn switch_theme(ctx: &Context, states: &mut States) {
    if !ctx.theme().default_visuals().dark_mode {
        states.theme_index += 1;
        if states.theme_index > 1 {
            states.theme_index = 0;
        }
        match states.theme_index {
            0 => {
                catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);
            }
            1 => {
                catppuccin_egui::set_theme(ctx, catppuccin_egui::LATTE);
            }
            _ => unreachable!(),
        }
    } else {
        states.theme_index += 1;
        if states.theme_index > 1 {
            states.theme_index = 0;
        }
        match states.theme_index {
            0 => {
                catppuccin_egui::set_theme(ctx, catppuccin_egui::MACCHIATO);
            }
            1 => {
                catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);
            }
            _ => unreachable!(),
        }
    }
}
