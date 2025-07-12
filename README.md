# rmon - Resource Monitor
![swappy-20250708_102027](https://github.com/user-attachments/assets/06faf67b-5cad-4c6c-931d-f1f56d47f623)

A lightweight CLI system monitor with real-time monitoring capabilities.

## Features

- **Real-time CPU monitoring** with per-core usage and temperatures
- **Memory usage tracking** with history graphs
- **Process display**
- **Disk usage monitoring** for root filesystem
- **Network activity monitoring** with download/upload rates
- **NVIDIA GPU statistics** (usage, temperature, fan, power, VRAM)
- **Journal listing**
- **Session-relative network totals**
- **Both TUI and simple text modes**
- **Comprehensive temperature monitoring**

![swappy-20250708_102056](https://github.com/user-attachments/assets/6b847023-c80a-4da4-9bd1-51228acf682c)

![swappy-20250708_102125](https://github.com/user-attachments/assets/694e6d10-cbe9-4c06-88d2-ad2f949a858c)

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
