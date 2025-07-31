mod components;
mod theming;

use std::{sync::{mpsc::{Receiver, Sender}, Arc, RwLock}};

use crate::{config::Config};

use anyhow::Result;
use eframe::{run_native, App, CreationContext, NativeOptions};

pub fn show_ui(config: Arc<RwLock<Config>>, sender: Sender<u8>, err: Receiver<anyhow::Error>) -> Result<()>{

    let native_options = NativeOptions { 
        ..Default::default()
    };
    run_native("minebak", native_options, Box::new(
        |cc: &CreationContext<'_> | {
            Ok(Box::new(MineBakApp::new(config, sender, err, cc)))
        }
    )).unwrap();
    Ok(())
}


pub struct MineBakApp{
    config: Arc<RwLock<Config>>,
    sender: Sender<u8>,
    err: Receiver<anyhow::Error>,
}

impl MineBakApp {
    pub fn new(config: Arc<RwLock<Config>>, sender: Sender<u8>, err: Receiver<anyhow::Error>, cc: &CreationContext<'_>) -> Self {
        theming::set_font(cc);
        Self { config, sender, err }
    }
}

impl App for MineBakApp{
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        components::draw_ui(ctx, &self.config, &self.sender, &self.err);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.sender.send(255).unwrap();
    }
}
