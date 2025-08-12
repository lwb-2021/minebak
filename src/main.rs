#![feature(duration_constructors_lite, path_add_extension)]
#![windows_subsystem = "windows"]

mod backup;
mod cloud_sync;
mod cmd;
mod config;
mod ui;
mod utils;

use std::{
    env,
    fs::{self, File},
    path::PathBuf,
    sync::{
        Arc, RwLock,
        mpsc::{self, Receiver},
    },
    thread::{self},
    time::Duration,
};

use anyhow::{Error, Result};
use backup::{MinecraftInstanceRoot, rescan_instances};
use clap::Parser;

use cloud_sync::run_sync;
use config::{default_backup_root, default_duration};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
use ui::Signal;

use crate::{backup::run_backup, ui::show_ui};

fn main() -> Result<()> {
    let arg = cmd::Args::parse();
    let config_path = if arg.config_path.is_some() {
        arg.config_path.unwrap()
    } else {
        let mut config_root =
            env::home_dir().expect("Cannot get home dir, please set --config-path");
        config_root.push(".minebak");
        config_root.push("config");
        config_root.push("config.ron");
        config_root
    };
    if !config_path.exists() {
        new_config(config_path.clone(), arg.backup_root.clone())?;
    }

    init_log()?;

    let mut res = config::read_config(config_path.clone());
    if res.is_err() {
        log::error!("Failed to read config: {:?}", res);
        notifica::notify("读配置失败，即将新建配置", "").unwrap();
        res = new_config(config_path.clone(), arg.backup_root.clone());
    }
    let mut configuration = res?;
    if arg.backup_root.is_some() {
        configuration.backup_root = arg.backup_root.unwrap();
    }
    if arg.duration.is_some() {
        configuration.duration = Duration::from_secs(arg.duration.unwrap());
    }
    

    for service in configuration.cloud_services.values_mut() {
        service.open_connection()?;
        log::debug!("Connection Opened for {:?}", service);
    }

    if configuration.autostart && !configuration.autostart_installed {
        configuration.autostart_installed = register_autostart()?;
    }

    run_sync(&configuration)?;
    if arg.run_backup {
        let _ = rescan_instances(&mut configuration).is_err_and(report_err_in_background);
        let _ = configuration
            .save(config_path)
            .is_err_and(report_err_in_background);
        if run_backup(&configuration)? {
            run_sync(&configuration)?;
        }
        return Ok(());
    }

    if arg.duration.is_some() {
        configuration.duration = Duration::from_secs(arg.duration.unwrap());
    }
    if arg.run_daemon {
        run_daemon(&mut configuration, config_path);
    }

    let configuration = Arc::new(RwLock::new(configuration));
    let configuration_clone = configuration.clone();
    let (sender, receiver) = mpsc::channel();
    let (error_reporter, error_receiver) = mpsc::channel();
    let logic_thread = thread::spawn(move || {
        while let Err(err) = run_logic(configuration.clone(), &receiver, config_path.clone()) {
            log::error!("Backend crashed: {}\n{}", err, err.backtrace());
            error_reporter.send(err).unwrap();
        }
    });
    let res = show_ui(configuration_clone, sender, error_receiver);
    if res.is_err() {
        log::error!("{}", res.unwrap_err())
    }
    logic_thread.join().unwrap();

    Ok(())
}

fn new_config(config_path: PathBuf, backup_root: Option<PathBuf>) -> Result<config::Config> {
    if config_path.exists() && config_path.is_file() {
        fs::copy(config_path.clone(), config_path.with_added_extension("bak"))?;
    }
    log::warn!("Creating new configuration");
    fs::create_dir_all(config_path.parent().unwrap())?;

    let backup_root = backup_root.unwrap_or_else(default_backup_root);
    if !backup_root.exists() {
        fs::create_dir_all(backup_root.clone())?;
    }
    let default_config = config::Config {
        duration: default_duration(),
        backup_root,
        ..Default::default()
    };

    default_config.save(config_path.clone()).unwrap();

    Ok(default_config)
}

fn report_err_in_background(err: Error) -> bool {
    log::error!("{}", err);
    notifica::notify("Minebak：备份出错", "详情请见日志").unwrap();
    Err::<(), anyhow::Error>(err).unwrap();
    true
}

#[allow(unused)]
fn run_daemon(configuration: &mut config::Config, config_path: PathBuf) -> ! {
    loop {
        rescan_instances(configuration).is_err_and(report_err_in_background);
        configuration
            .save(config_path.clone())
            .is_err_and(report_err_in_background);
        run_backup(&configuration)
            .map(|res| {
                if res {
                    run_sync(&configuration);
                }
            })
            .is_err_and(report_err_in_background);
        thread::sleep(configuration.duration);
    }
}

fn run_logic(
    configuration: Arc<RwLock<config::Config>>,
    receiver: &Receiver<Signal>,
    config_path: PathBuf,
) -> Result<()> {
    loop {
        if let Result::Ok(signal) = receiver.recv_timeout(Duration::from_secs(1)) {
            match signal {
                Signal::Rescan => {
                    backup::rescan_instances(&mut configuration.try_write().unwrap())?;
                }
                Signal::Exit => {
                    configuration.read().unwrap().save(config_path.clone())?;
                    break;
                }
                Signal::RunBackup => {
                    if run_backup(&configuration.read().unwrap())? {
                        run_sync(&configuration.read().unwrap())?;
                    }
                }
                Signal::AddInstance {
                    name,
                    path,
                    multimc,
                    version_isolated,
                } => {
                    configuration
                        .write()
                        .unwrap()
                        .instance_roots
                        .push(MinecraftInstanceRoot::new(
                            name,
                            path,
                            multimc,
                            version_isolated,
                        )?);
                    configuration.read().unwrap().save(config_path.clone())?;
                }
                Signal::Recover { save, timestamp } => {
                    save.recover(configuration.read().unwrap().backup_root.clone(), timestamp)?;
                }
                #[allow(unreachable_patterns)]
                s => todo!("{:?}", s),
            }
        }
    }
    Ok(())
}

fn init_log() -> Result<()> {
    let mut log_file_path = env::temp_dir();
    log_file_path.push("minebak.log");
    let encoder = Box::new(PatternEncoder::new("{l} - {m}\n"));
    File::create(&log_file_path)?;
    let log_file_appender = FileAppender::builder()
        .encoder(encoder.clone())
        .build(log_file_path)?;

    let std_out_appender = ConsoleAppender::builder().encoder(encoder).build();

    #[allow(unused)]
    let level = log::LevelFilter::Info;

    #[cfg(debug_assertions)]
    let level = log::LevelFilter::Debug;

    let log4rs_config = log4rs::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(log_file_appender)))
        .appender(Appender::builder().build("std_out", Box::new(std_out_appender)))
        .build(
            Root::builder()
                .appender("logfile")
                .appender("std_out")
                .build(level),
        )?;
    log4rs::init_config(log4rs_config)?;
    Ok(())
}

#[cfg(target_os="linux")]
const DESKTOP_FILE_AUTOSTART: &[u8; 61] = include_bytes!("../resources/autostart.desktop");
#[cfg(target_os="linux")]
fn register_autostart() -> Result<bool>{
    log::info!("Trying to register autostart");
    if let Some(mut home) = env::home_dir() {
        home.push(".config");
        home.push("autostart");
        if home.exists() {
            home.push("minebak.desktop");
            fs::write(&home, DESKTOP_FILE_AUTOSTART)?;
        }
        log::info!("Registered autostart in {:?}", home);
        return Ok(true);
    }
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        use anyhow::Ok;

        let mut config_home = PathBuf::from(config_home);
        if config_home.exists() {
            config_home.push("autostart");
            if !config_home.exists() {
                fs::create_dir(&config_home)?
            }
            fs::write(&config_home, DESKTOP_FILE_AUTOSTART)?;
        }
        log::info!("Registered autostart in {:?}", config_home);
        return Ok(true);
    }

    Ok(false)
}

#[cfg(target_os="windows")]
fn register_cron() -> Result<()> {
    todo!()
}