use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use crate::conf::PAYLOADS;
use crate::Response;
use crate::utils::{get_payload, write_to_file};

/// Scans a single TCP port with logging enabled.
/// Connects to the target address and port within a timeout,
/// sends a predefined payload, reads the response,
/// and logs the results including any errors.
///
/// # Arguments
/// * `_addr` - Target IP address as a `String`
/// * `port` - Target port number
/// * `conn_timeout` - Timeout in milliseconds for connection and I/O operations
async fn scan_single_port(_addr: String, port: u16, conn_timeout: u64){
    let addr = _addr.as_str();
    println!("Scanning single port: {addr}:{port}");

    let socket_addr = format!("{}:{}", addr, port);

    // Attempt to connect to the target socket with timeout
    let connect_result = timeout(
        Duration::from_millis(conn_timeout),
        TcpStream::connect(&socket_addr),
    ).await;

    let mut stream = match connect_result {
        Ok(Ok(s)) => s, // Successfully connected
        Ok(Err(e)) => {
            eprintln!("[error] Connect error to {}: {}", socket_addr, e);
            return;
        }
        Err(_) => {
            eprintln!("[error] Connect timeout to {}", socket_addr);
            return;
        }
    };

    // Retrieve the payload to send based on the port
    let mut payload = String::new();
    match get_payload(port).await {
        Ok(_payload) => payload = _payload,
        Err(_) => {
            eprintln!("[critical] Payload for port {port} not found");
        }
    }

    // Send payload and read response with timeout
    let io_result = timeout(Duration::from_millis(conn_timeout), async {
        stream.write_all(&payload.as_bytes()).await?;
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await?;
        buf.truncate(n);
        Ok::<_, std::io::Error>(buf)
    }).await;

    // Handle read/write results and errors
    let response_bytes = match io_result {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => {
            eprintln!("[error] IO error on {}: {}", socket_addr, e);
            return;
        }
        Err(_) => {
            eprintln!("[error] IO timeout on {}", socket_addr);
            return;
        }
    };

    // Ignore empty or whitespace-only responses
    if response_bytes.trim_ascii().is_empty() {
        return;
    }

    // Convert response bytes to UTF-8 string
    let response_str = String::from_utf8_lossy(&response_bytes).to_string();

    // If response is non-empty, create a Response struct, print and log it
    if !response_str.trim().is_empty() {
        let resp = Response {
            port,
            addr: socket_addr.clone(),
            payload,
            response: response_str,
        };
        println!("[{}] {}", port, resp.addr);
        write_to_file(&resp).await;
    }
}

/// Scans a single TCP port without logging or printing errors.
/// Similar to `scan_single_port` but silently ignores errors.
///
/// # Arguments
/// * `_addr` - Target IP address as a `String`
/// * `port` - Target port number
/// * `conn_timeout` - Timeout in milliseconds for connection and I/O operations
async fn scan_single_port_no_log(_addr: String, port: u16, conn_timeout: u64){
    let addr = _addr.as_str();

    let socket_addr = format!("{}:{}", addr, port);

    // Attempt to connect to the target socket with timeout
    let connect_result = timeout(
        Duration::from_millis(conn_timeout),
        TcpStream::connect(&socket_addr),
    ).await;

    let mut stream = match connect_result {
        Ok(Ok(s)) => s,
        Ok(Err(_)) => {
            return; // silently ignore errors
        }
        Err(_) => {
            return; // silently ignore timeouts
        }
    };

    // Retrieve payload for the given port
    let mut payload = String::new();
    match get_payload(port).await {
        Ok(_payload) => payload = _payload,
        Err(_) => {
            eprintln!("[critical] Payload for port {port} not found");
        }
    }

    // Send payload and read response with timeout
    let io_result = timeout(Duration::from_millis(conn_timeout), async {
        stream.write_all(&payload.as_bytes()).await?;
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await?;
        buf.truncate(n);
        Ok::<_, std::io::Error>(buf)
    }).await;

    // Silently ignore IO errors or timeouts
    let response_bytes = match io_result {
        Ok(Ok(data)) => data,
        Ok(Err(_)) => {
            return;
        }
        Err(_) => {
            return;
        }
    };

    // Skip empty responses
    if response_bytes.trim_ascii().is_empty() {
        return;
    }

    // Convert response bytes to UTF-8 string
    let response_str = String::from_utf8_lossy(&response_bytes).to_string();

    // If response is non-empty, create Response and write it to file
    if !response_str.trim().is_empty() {
        let resp = Response {
            port,
            addr: socket_addr.clone(),
            payload,
            response: response_str,
        };
        write_to_file(&resp).await;
    }
}

/// Scans all configured ports on a given IP address concurrently.
///
/// # Arguments
/// * `addr` - Target IP address as `String`
/// * `timeout` - Timeout in milliseconds for each individual port scan
/// * `logging` - Whether to log detailed scan output or run silently
pub async fn scan_ip_with_ports(addr: String, timeout: u64, logging: bool) {
    // Spawn a Tokio task for each port defined in the PAYLOADS map
    let tasks: Vec<_> = PAYLOADS.read().await.keys().map(|&port| {
        let ip = addr.clone();
        tokio::spawn(async move {
            if logging {
                scan_single_port(ip, port, timeout).await; // scan with logging
            } else {
                scan_single_port_no_log(ip, port, timeout).await; // scan silently
            }
        })
    }).collect();

    // Await all port scan tasks to complete
    for task in tasks {
        let _ = task.await;
    }
}
