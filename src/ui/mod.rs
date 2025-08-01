mod components;
mod theming;

use std::{path::PathBuf, sync::{mpsc::{Receiver, Sender}, Arc, RwLock}};

use crate::{config::Config};

use anyhow::Result;
use eframe::{run_native, App, CreationContext, NativeOptions};

#[derive(Debug, Clone)]
pub enum Signal {
    Rescan,
    RunBackup,
    AddInstance {
        name: String,
        path: PathBuf,
        multimc: bool,
        version_isolated: bool
    },
    Recover(),
    Exit
}

#[derive(Debug, Default)]
pub struct States {
    window_add_save_show: bool,
    add_save_window_error_message: String,
    add_save_window_path_input: String,
    add_save_window_name_input: String,
}

pub fn show_ui(config: Arc<RwLock<Config>>, sender: Sender<Signal>, err: Receiver<anyhow::Error>) -> Result<()>{

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
    sender: Sender<Signal>,
    states: States,
    err: Receiver<anyhow::Error>,
}

impl MineBakApp {
    pub fn new(config: Arc<RwLock<Config>>, sender: Sender<Signal>, err: Receiver<anyhow::Error>, cc: &CreationContext<'_>) -> Self {
        theming::set_font(cc);
        let mut app = Self { config, sender, err, states: States::default() };
        app
    }
}

impl App for MineBakApp{
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        components::draw_ui(ctx, self);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.sender.send(Signal::Exit).unwrap();
    }
}
