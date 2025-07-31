use std::sync::Arc;

use eframe::{egui::{self, FontData, FontTweak}, CreationContext};



pub(super) fn set_font(cc: &CreationContext<'_>){
    let mut fonts = egui::FontDefinitions::default();

    let font_tweak = FontTweak { 
        ..Default::default() 
    };

    fonts.font_data.insert(
        "Source Han Sans SC".to_string(), 
        Arc::new(
            FontData::from_static(include_bytes!("../../resources/SourceHanSansCN-VF.otf"))
            .tweak(font_tweak)
        )
    );
    
    let proportional = fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap();
    proportional.insert(0, "Source Han Sans SC".to_string());
    cc.egui_ctx.set_fonts(fonts);
    
}
