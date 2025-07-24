use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use std::process::exit;
use std::sync::atomic::{AtomicBool, AtomicU64};
use anyhow::Context;
use tokio::sync::{Mutex, RwLock};
use serde_json;
use serde::Deserialize;
use clap;
use clap::Parser;
use once_cell::sync::Lazy;
use serde_json::Value;

pub static FILE_LOGGING: AtomicBool = AtomicBool::new(false);
pub static CONSOLE_LOGGING: AtomicBool = AtomicBool::new(false);
pub static TASK: AtomicU64 = AtomicU64::new(0);
pub static PAYLOADS: Lazy<RwLock<HashMap<u16, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));
pub static TIMEOUT: AtomicU64 = AtomicU64::new(0);

#[derive(Deserialize)]
pub struct Payloads {
    pub payloads: HashMap<u16, String>,
}

#[derive(clap::Parser)]
pub struct Args {
    #[arg(short = 'n', long = "tasks_max", default_value = "3000")] // количество задач
    tasks: u64,

    #[arg(short = 'l', long = "log_file", default_value = "logs.csv")] // имя файла
    log_file: String,

    #[arg(short = 'f', long = "file_logging", default_value_t = false)] // логировать в файл
    file_logging: bool,

    #[arg(short = 'c', long = "console_logging", default_value_t = true)] // логировать в консоль
    console: bool,

    #[arg(short = 't', long = "timeout", default_value_t = 700)] // таймаут подключения
    timeout: u64,

    #[arg(short = 'p', long = "payloads_file", default_value = "payloads.json")]
    payloads_filename: String,
}

pub struct ArgsConfig {
    pub log_file: String,
    pub payloads_filename: String,
}



pub async fn read_payloads_json(config_file: &str) -> anyhow::Result<()> {
    let path = Path::new(config_file);
    if !path.exists() {
        eprintln!("Config file {} does not exist", config_file);
        exit(1);
    }

    let mut file = OpenOptions::new().read(true).open(config_file)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    // using Value for parsing
    let json: Value = serde_json::from_slice(&buf)?;
    let obj = json.as_object()
        .context("Payload config must be a JSON object")?;

    for (key, value) in obj {
        if key.starts_with('_') {
            continue; // skip comments
        }

        let port: u16 = key.parse()
            .with_context(|| format!("Invalid port key: '{}'", key))?;

        let payload = value.as_str()
            .context(format!("Expected string payload for port {}", port))?
            .to_string();

        PAYLOADS.write().await.insert(port, payload);
    }

    Ok(())
}

pub async fn read_clap_conf() -> ArgsConfig {
    let args = Args::parse();

    CONSOLE_LOGGING.store(args.console, std::sync::atomic::Ordering::Relaxed);
    FILE_LOGGING.store(args.file_logging,std::sync::atomic::Ordering::Relaxed);
    TASK.store(args.tasks, std::sync::atomic::Ordering::Relaxed);
    TIMEOUT.store(args.timeout, std::sync::atomic::Ordering::Relaxed);

    ArgsConfig {
        log_file: args.log_file,
        payloads_filename: args.payloads_filename,
    }

}
