# rmon - Resource Monitor

A lightweight CLI system monitor with real-time monitoring capabilities.

## Features

- **Real-time CPU monitoring** with per-core usage and temperatures
- **Memory usage tracking** with history graphs
- **Process display**
- **Disk usage monitoring** for root filesystem
- **Network activity monitoring** with download/upload rates
- **Journal listing**
- **Session-relative network totals**
- **Both TUI and simple text modes**
- **Comprehensive temperature monitoring**

## Installation

### From Packages

#### DEB Package (Debian/Ubuntu)
#### RPM Package (RHEL/CentOS/Fedora)
- Download from releases section

### From Source

#### Prerequisites
- Rust 1.70+ 
- Cargo

#### Build and Install
```bash
git clone https://github.com/codedsleep/rmon
cd rmon
cargo build --release
sudo cp target/release/rmon /usr/local/bin/
```

## Usage

### TUI Mode (default)
```bash
rmon
```

### Simple Text Mode
```bash
rmon --simple
```

### Navigation (TUI Mode)
- **Tab**: Switch between panels (System Monitor, Processes, Journal Logs)
- **↑/↓**: Scroll in lists
- **PgUp/PgDn**: Fast scroll
- **C**: Sort processes by CPU usage
- **M**: Sort processes by Memory usage
- **K**: Kill selected process
- **q/Ctrl+C**: Quit

````
