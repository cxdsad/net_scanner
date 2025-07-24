use std::backtrace::Backtrace;
use std::panic;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use csv_async::AsyncSerializer;
use tokio::fs::OpenOptions;
use tokio::sync::Mutex;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use crate::conf::*;
use crate::{Response, LOGGER};


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

    LOGGER.set(Arc::new(Mutex::new(AsyncSerializer::from_writer(file.compat_write()))))
        .expect("Logger already initialized");
}

pub async fn write_to_file(data: &Response) {
    if !FILE_LOGGING.load(Ordering::Relaxed) {
        return;
    }
    let writer = LOGGER.get().expect("Logger not initialized").clone();
    let mut writer = writer.lock().await;

    if let Err(e) = writer.serialize(data).await {
        eprintln!("Write error: {e}");
    }
}

pub async fn get_payload(port: u16) -> anyhow::Result<String> {
    if let Some(payloads) = PAYLOADS.read().await.get(&port) {
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
            Help    : Please open an issue on GitHub\n",
            payload,
            location,
        );
    }));
}