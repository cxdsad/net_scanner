use std::panic;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use csv_async::AsyncSerializer;
use tokio::fs::OpenOptions;
use tokio::io::BufWriter;
use tokio::sync::Mutex;
use tokio_util::compat::TokioAsyncReadCompatExt;
use crate::conf::*;
use crate::{Response, LOGGER};


static WRITE_COUNT: AtomicUsize = AtomicUsize::new(0);

pub async fn init_logger(filename: &str) {
    if !FILE_LOGGING.load(Ordering::Relaxed) {
        return;
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .await
        .expect("Failed to open log file");

    let buf_writer = BufWriter::new(file);

    LOGGER.set(Arc::new(Mutex::new(AsyncSerializer::from_writer(buf_writer.compat()))))
        .expect("Logger already initialized");
}

pub async fn write_to_file(data: &Response) {
    if !FILE_LOGGING.load(Ordering::Relaxed) {
        return;
    }

    let logger = LOGGER.get().expect("Logger not initialized").clone();
    let mut serializer = logger.lock().await;

    if let Err(e) = serializer.serialize(data).await {
        eprintln!("Write error: {e}");
        return;
    }

    let count = WRITE_COUNT.fetch_add(1, Ordering::Relaxed) + 1;

    if count % 10 == 0 {
        if let Err(e) = serializer.flush().await {
            eprintln!("Flush error: {e}");
        }
    }
}

pub async fn get_payload(port: u16, ip: &str) -> anyhow::Result<String> {
    if let Some(payloads) = PAYLOADS.read().await.get(&port) {
        if PLACEHOLDERS.load(Ordering::Relaxed) {
            return Ok(payloads.clone().replace("%ip%", ip).replace("%port%", port.to_string().as_str()));
        }

        return Ok(payloads.clone());
    }
    else {
        eprintln!("No such port: {}", port);
        return Err(anyhow::anyhow!(format!("{port} is not registered")));
    }
}

pub fn set_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let payload = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => s.as_str(),
                None => "Unknown panic",
            },
        };

        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());

        eprintln!(
            "\n\
            \x1b[31m[!] Scanner Panicked\x1b[0m\n\
            Message : {}\n\
            Location: {}\n\
            Help    : Please open an issue on GitHub: https://github.com/cxdsad/net_scanner/issues\n",
            payload,
            location,
        );
    }));
}