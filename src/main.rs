use csv_async::AsyncSerializer;
use rand::Rng;
use serde::Serialize;
use std::net::Ipv4Addr;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio::io::{BufReader, BufWriter, AsyncBufReadExt, AsyncWriteExt};
use tokio_util::compat::Compat;
use crate::conf::*;
use crate::scan::scan_ip_with_ports;
use crate::utils::{init_logger, set_panic_hook};

mod scan;
mod conf;
mod utils;

// Global async CSV logger protected by a mutex and initialized once
static LOGGER: OnceLock<Arc<Mutex<AsyncSerializer<Compat<BufWriter<tokio::fs::File>>>>>> = OnceLock::new();

// Atomic counter tracking the number of currently active scanning tasks
static ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(0);

/// Struct representing the scanning response, serialized for logging or output
#[derive(Serialize)]
pub struct Response {
    pub port: u16,       // The scanned port number
    pub addr: String,    // Target IP address
    pub payload: String, // Sent payload data
    pub response: String,// Received response from the target
}

/// Generates a random IPv4 address excluding reserved private and localhost ranges.
///
/// Excluded ranges:
/// - 10.0.0.0/8
/// - 127.0.0.0/8 (localhost)
/// - 192.168.0.0/16
/// - 172.16.0.0/12
fn random_ip() -> String {
    let mut rng = rand::thread_rng();
    loop {
        // Generate each octet within valid public IP ranges
        let oct1 = rng.gen_range(1..=223);
        let oct2 = rng.gen_range(0..=255);
        let oct3 = rng.gen_range(0..=255);
        let oct4 = rng.gen_range(1..=254);

        // Skip private and localhost ranges
        if oct1 == 10
            || oct1 == 127
            || (oct1 == 192 && oct2 == 168)
            || (oct1 == 172 && (16..=31).contains(&oct2))
        {
            continue;
        }

        let ip = Ipv4Addr::new(oct1, oct2, oct3, oct4);
        return ip.to_string();
    }
}

/// Main asynchronous entry point.
///
/// Sets up the logger, reads configuration and payloads,
/// and enters an infinite loop spawning scanning tasks up to a configured maximum.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set panic hook for better error reporting with backtraces
    set_panic_hook();

    // Parse CLI arguments into configuration
    let args_conf = read_clap_conf().await;

    // Initialize the file logger if enabled
    init_logger(args_conf.log_file.as_str()).await;

    // Load payloads from JSON file for scanning
    read_payloads_json(args_conf.payloads_filename.as_str()).await?;

    // Load global atomic configuration parameters
    let max_tasks = TASK.load(Relaxed);
    let timeout = TIMEOUT.load(Relaxed);
    let console_logging = CONSOLE_LOGGING.load(Relaxed);

    // Print current configuration to console
    println!("tasks: {}", max_tasks);
    println!("time: {}", timeout);
    println!("console_logging: {}", console_logging);
    println!("file_logging: {}", FILE_LOGGING.load(Relaxed));

    // Container for managing concurrent scanning tasks
    let mut set = JoinSet::new();

    // Infinite scanning loop
    loop {
        // Spawn new scanning tasks if below max_tasks limit
        while ACTIVE_TASKS.load(Relaxed) < max_tasks as usize {
            let ip = random_ip();
            ACTIVE_TASKS.fetch_add(1, Relaxed);

            // Spawn an async task to scan the generated IP
            set.spawn(async move {
                scan_ip_with_ports(ip, timeout, console_logging).await;
                ACTIVE_TASKS.fetch_sub(1, Relaxed);
            });
        }

        // Poll and handle finished tasks, logging any errors
        while let Some(res) = set.try_join_next() {
            if let Err(e) = res {
                eprintln!("Task failed: {e}");
            }
        }
    }
}

/// A simple test to verify scanning the localhost IP (127.0.0.1).
#[tokio::test]
async fn test_localhost() -> anyhow::Result<()> {
    scan_ip_with_ports("127.0.0.1".to_string(), 400, true).await;
    Ok(())
}
