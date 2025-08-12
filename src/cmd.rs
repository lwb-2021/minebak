use std::path::PathBuf;

use clap::Parser;
#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[arg(short='b', long="run-backup", help="Run backup in background")]
    pub run_backup: bool,

    #[arg(short='d', long="run-daemon", help="Run daemon in background")]
    pub run_daemon: bool,
    
    #[arg(short='c', long="config-path")]
    pub config_path: Option<PathBuf>,

    #[arg(long="backup-root")]
    pub backup_root: Option<PathBuf>,

    #[arg(long="duration", help="Backup duration (second)")]
    pub duration: Option<u64>
}
