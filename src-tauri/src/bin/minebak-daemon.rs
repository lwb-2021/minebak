use clap::Parser;
use std::{
    io::{Read, Result, Write},
    net::{TcpListener, TcpStream},
    time::{Duration, SystemTime},
};

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(default_value_t = 30)]
    pub backup_duration: u64,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let server = TcpListener::bind("127.0.0.1:25569")?;
    let mut earlier = SystemTime::now();
    loop {
        for stream in server.incoming() {
            if let Ok(mut stream) = stream
                && let Err(err) = handle_request(&mut stream)
            {
                let _ = write!(stream, "Error: {:?}", err);
                let _ = stream.flush();
                log::error!("Failed to handle connection: {:?}", err)
            }
        }
        if SystemTime::now().duration_since(earlier).unwrap()
            >= Duration::from_mins(cli.backup_duration)
        {
            earlier = SystemTime::now()
        }
    }
}
fn handle_request(stream: &mut TcpStream) -> Result<()> {
    let mut id: [u8; 1] = [0];
    stream.read_exact(&mut id)?;
    match id[0] {
        0 => backup(stream)?,
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unexpected command {:?}", id),
            ));
        }
    }
    stream.flush()
}

fn backup(stream: &mut TcpStream) -> Result<()> {
    if std::env::temp_dir().join("minebak-backup.lock").exists() {
        write!(stream, "Waiting for Lock")?;
        log::info!("Waiting for Lock")
    }
    stream.flush()
}
