#![feature(duration_constructors_lite, path_add_extension)]
#![windows_subsystem = "windows"]

mod backup;
mod cmd;
mod config;
mod ui;

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

use anyhow::{Error, Ok, Result};
use backup::{MinecraftInstanceRoot, rescan_instances};
use clap::Parser;

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
        fs::create_dir_all(config_path.parent().unwrap())?;
        let mut default_config = config::Config::default();
        default_config.duration = Duration::from_hours(1);

        let mut minebak_root = config_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        minebak_root.push("backup");
        if !minebak_root.exists() {
            fs::create_dir_all(minebak_root.clone())?;
        }
        default_config.backup_root = minebak_root;
        default_config.save(config_path.clone()).unwrap();
    }

    init_log()?;
    let mut configuration = config::read_config(config_path.clone())?;

    if arg.duration.is_some() {
        configuration.duration = Duration::from_secs(arg.duration.unwrap());
    }
    if arg.run_backup {
        let _ = rescan_instances(&mut configuration).is_err_and(report_err_in_background);
        let _ = configuration
            .save(config_path)
            .is_err_and(report_err_in_background);
        return run_backup(&configuration);
    }
    if arg.run_daemon {
        run_daemon(&mut configuration, config_path);
    }

    let configuration = Arc::new(RwLock::new(configuration));
    let configuration_clone = configuration.clone();
    let (sender, receiver) = mpsc::channel();
    let (error_reporter, error_receiver) = mpsc::channel();
    let logic_thread = thread::spawn(move || {
        let res = run_logic(configuration, receiver, config_path);
        if res.is_err() {
            error_reporter.send(res.unwrap_err()).unwrap();
        }
    });
    let res = show_ui(configuration_clone, sender, error_receiver);
    if res.is_err() {
        log::error!("{}", res.unwrap_err())
    }
    logic_thread.join().unwrap();

    Ok(())
}

fn report_err_in_background(err: Error) -> bool {
    log::error!("TODO: report error");
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
        run_backup(&configuration).is_err_and(report_err_in_background);
        thread::sleep(configuration.duration);
    }
}

fn run_logic(
    configuration: Arc<RwLock<config::Config>>,
    receiver: Receiver<Signal>,
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
                    run_backup(&configuration.read().unwrap())?;
                }
                Signal::AddInstance {
                    name,
                    path,
                    multimc,
                    version_isolated,
                } => {
                    configuration.write().unwrap().instance_roots.push(
                        MinecraftInstanceRoot::new(name, path, multimc, version_isolated)?
                    );
                    configuration.read().unwrap().save(config_path.clone())?;
                }
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
