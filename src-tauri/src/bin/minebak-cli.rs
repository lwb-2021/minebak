use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(subcommand)]
    Backup(BackupSubcommand),
    #[clap(subcommand)]
    Instance(InstanceSubcommand),
}

#[derive(Debug, Subcommand)]
pub enum InstanceSubcommand {}

#[derive(Debug, Subcommand)]
pub enum BackupSubcommand {
    Start {},
    List { instance: String },
}

fn process_backup(subcommand: BackupSubcommand) -> ExitCode {
    match subcommand {
        #[allow(unreachable_patterns)]
        _ => todo!(),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Backup(subcommand) => process_backup(subcommand),
        #[allow(unreachable_patterns)]
        _ => todo!(),
    }
}
