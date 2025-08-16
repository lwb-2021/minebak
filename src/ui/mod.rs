mod components;
mod theming;

use std::{
    path::PathBuf, sync::{
        mpsc::{Receiver, Sender}, Arc, RwLock, RwLockReadGuard
    }, time::Duration
};

use crate::{backup::MinecraftSave, config::Config};

use anyhow::Result;
use eframe::{App, CreationContext, NativeOptions, run_native};

#[derive(Debug, Clone)]
pub enum Signal {
    Rescan,
    RunBackup,
    AddInstance {
        name: String,
        path: PathBuf,
        multimc: bool,
        version_isolated: bool,
    },
    Recover {
        save: MinecraftSave,
        timestamp: String,
    },
    SaveConfig,
    Exit,
}
#[derive(Default, Debug)]
struct AppSettings {
    autostart: bool,
    cron: bool,
    backup_duration_mins: u64,
    backup_root: String,
}

impl<'a> From<&'a RwLockReadGuard<'a, Config>> for AppSettings {
    fn from(value: &'a RwLockReadGuard<'a, Config>) -> Self {
        AppSettings {
            autostart: value.autostart,
            cron: value.cron,
            backup_duration_mins: value.duration.as_secs_f64() as u64 / 60,
            backup_root: value.backup_root.to_string_lossy().to_string()
        }
    }
}

impl AppSettings {
    pub fn save(&mut self, config: &mut Config) {
        config.autostart = self.autostart;
        config.cron = self.cron;
        config.duration = Duration::from_mins(self.backup_duration_mins);
        config.backup_root = PathBuf::from(self.backup_root.clone());
    }
}

#[derive(Debug, Default)]
pub struct States {
    theme_index: u8,

    central_panel: u8,

    window_add_save_show: bool,
    add_save_window_error_message: String,
    add_save_window_path_input: String,
    add_save_window_name_input: String,

    window_recover_show: bool,
    window_recover_refreshed: bool,
    recover_current_save: Option<MinecraftSave>,
    recover_backup_names: Vec<String>,

    window_settings_show: bool,
    settings: AppSettings,
    webdav_window_open: bool,
    webdav_window_err: String,

    webdav_endpoint: String,
    webdav_username: String,
    webdav_password: String,

    rclone_window_open: bool,
    rclone_remote: String,

    save_edit_window_open: bool,
    edit_save_index: [usize;3],
    edit_save_info: MinecraftSave,

    err_list: Vec<anyhow::Error>,
}

pub fn show_ui(
    config: Arc<RwLock<Config>>,
    sender: Sender<Signal>,
    err: Receiver<anyhow::Error>,
) -> Result<()> {
    let native_options = NativeOptions {
        ..Default::default()
    };
    run_native(
        "minebak",
        native_options,
        Box::new(|cc: &CreationContext<'_>| Ok(Box::new(MineBakApp::new(config, sender, err, cc)))),
    )
    .unwrap();
    Ok(())
}

pub struct MineBakApp {
    config: Arc<RwLock<Config>>,
    sender: Sender<Signal>,
    states: States,
    err: Receiver<anyhow::Error>,
}

impl MineBakApp {
    pub fn new(
        config: Arc<RwLock<Config>>,
        sender: Sender<Signal>,
        err: Receiver<anyhow::Error>,
        cc: &CreationContext<'_>,
    ) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        theming::set_font(cc);
        let mut app = Self {
            config,
            sender,
            err,
            states: States::default(),
        };
        app.states.settings = AppSettings::from(&app.config.read().unwrap());
        app
    }
}

impl App for MineBakApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        components::draw_ui(ctx, self);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.sender.send(Signal::Exit).unwrap();
    }
}
