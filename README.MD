# Async Net Scanner

A high-performance asynchronous TCP port scanner written in Rust using `tokio`, capable of scanning random public IPs with payload-based interaction. It supports custom payloads per port and logs meaningful responses either to the console or to a CSV file.

---
## ✨ Features

- Asynchronous, non-blocking design with tokio
- Randomized public IPv4 scanning (excluding private/reserved ranges)
- Custom payloads per port loaded from a JSON file
- CSV logging via csv_async (optional)
- Console logging toggle
- Configurable task limits and timeouts
- Graceful panic handling with stack traces
- Concurrent multi-port scanning per host
- Placeholders (%ip% and %port%)

---


### 📄 `payloads.json` Format

The `payloads.json` file defines custom payloads to be sent for each port during scanning. This allows you to elicit meaningful responses from common services.

#### 🧾 Example

```json
{
  "_comment": "This is a comment and will be ignored by the parser",
  "80":  "GET / HTTP/1.1\\r\\nHost: %ip%:%port%\\r\\n\\r\\n",
  "21":  "USER anonymous\\r\\nPASS guest@\\r\\n",
  "443": "HEAD / HTTP/1.0\\r\\n\\r\\n"
}
```
---
### 🛠️ Args


| Option                                    | Description                        | Default       |
|-------------------------------------------|------------------------------------|---------------|
| `-n, --tasks_max <TASKS>`                 | Maximum number of concurrent tasks | 3000          |
| `-l, --log_file <LOG_FILE>`               | CSV log file name                  | logs.csv      |
| `-f, --file_logging`                      | Enable logging to file             | Disabled      |
| `-c, --console_logging`                   | Enable logging to console          | Disables      |
| `-t, --timeout <TIMEOUT>`                 | Connection and I/O timeout (ms)    | 700           |
| `-p, --payloads_file <PAYLOADS_FILENAME>` | Path to payloads JSON file         | payloads.json |
| `-h, --help`                              | Print help message                 | N/A           |
---
## Installation

1. Download the ZIP archive for your operating system from the [Releases](#) page.
2. Extract the contents of the ZIP archive to a folder of your choice.
3. Open a terminal or command prompt and navigate to the extracted folder.
4. Run the scanner executable with the desired command-line arguments.

linux (example):
```bash
chmod +x ./net_scanner
```
```bash
./net_scanner -n 1000 -c -p payloads.json
```

windows (example):
```cmd
net_scanner.exe -n 1000 -c -p payloads.json
```
