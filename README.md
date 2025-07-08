# rmon - Resource Monitor

A lightweight CLI system monitor with real-time monitoring capabilities.

## Features

- **Real-time CPU monitoring** with per-core usage and temperatures
- **Memory usage tracking** with history graphs
- **Disk usage monitoring** for root filesystem
- **Network activity monitoring** with download/upload rates
- **Session-relative network totals**
- **Both TUI and simple text modes**
- **Comprehensive temperature monitoring**

## Installation

### From Packages

#### DEB Package (Debian/Ubuntu)
```bash
# Download the latest release
wget https://github.com/example/rmon/releases/latest/download/rmon_0.1.0-1_amd64.deb

# Install
sudo dpkg -i rmon_0.1.0-1_amd64.deb

# If dependencies are missing:
sudo apt-get install -f
```

#### RPM Package (RHEL/CentOS/Fedora)
```bash
# Download the latest release
wget https://github.com/example/rmon/releases/latest/download/rmon-0.1.0-1.x86_64.rpm

# Install
sudo rpm -i rmon-0.1.0-1.x86_64.rpm
# or with yum/dnf:
sudo yum install rmon-0.1.0-1.x86_64.rpm
```

### From Source

#### Prerequisites
- Rust 1.70+ 
- Cargo

#### Build and Install
```bash
git clone https://github.com/example/rmon
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
