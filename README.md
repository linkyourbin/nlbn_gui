# NLBN GUI - Neat Library Bring Now

<div align="center">

![NLBN Logo](src/assets/nlbn.svg)

**A fast and intuitive desktop application for converting EasyEDA/LCSC components to KiCad format**

[![Release](https://img.shields.io/github/v/release/linkyourbin/nlbn_gui)](https://github.com/linkyourbin/nlbn_gui/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Built with Tauri](https://img.shields.io/badge/Built%20with-Tauri-FFC131?logo=tauri)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-orange?logo=rust)](https://www.rust-lang.org)

[English](#) | [中文](#中文说明)

</div>

## ✨ Features

- 🔄 **Single & Batch Conversion** - Convert individual components or process multiple LCSC IDs at once
- 📊 **Real-time Progress Tracking** - Visual progress updates showing current conversion status (n/m format)
- 📁 **File Import** - Import LCSC IDs from text files (supports comma, space, and newline delimiters)
- 📝 **Conversion History** - Scrollable history showing recent conversions with timestamp
- ✅ **Smart Error Reporting** - Only displays failed components, successful ones automatically go to history
- ⚙️ **Flexible Output Options** - Choose what to convert: Symbol, Footprint, and/or 3D Model
- 🎨 **Clean UI Design** - Compact and intuitive interface with optimal 1130x530 window size
- 🚀 **Native Performance** - Built with Rust backend for fast conversion

## 📥 Download & Installation

### Windows

Download the latest version from [Releases](https://github.com/linkyourbin/nlbn_gui/releases/latest):

| Installer Type | Size | Description |
|---------------|------|-------------|
| **NSIS Installer** (Recommended) | 4.6 MB | Windows installer with setup wizard and start menu shortcuts |
| **MSI Installer** | 6.5 MB | Standard Windows installer, good for enterprise deployment |
| **Portable EXE** | 18 MB | No installation required, run directly from USB or any folder |

**System Requirements:**
- Windows 10/11 (x64)
- Internet connection (for fetching component data from LCSC)

## 🚀 Quick Start

1. Download and install NLBN GUI
2. Launch the application
3. Enter an LCSC ID (e.g., `C5149201` for STM32G431CBU6)
4. Select output directory
5. Choose conversion options (Symbol/Footprint/3D Model)
6. Click "Convert" or use batch mode for multiple components

### Batch Conversion

- **Manual Entry**: Enter multiple LCSC IDs separated by commas or spaces
- **File Import**: Click "Import from File" to load IDs from a text file

Example input formats:
```
C5149201, C5149202, C5149203
C5149201 C5149202 C5149203
C5149201
C5149202
C5149203
```

## 🖼️ Screenshots

### Main Interface
*Clean and intuitive UI with real-time progress tracking*

### Batch Conversion
*Process multiple components with live progress updates*

### Conversion History
*Track all your conversions with searchable history*

## 🛠️ Development

### Prerequisites

- [Node.js](https://nodejs.org) (v18 or later)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Tauri Prerequisites](https://tauri.app/v2/guides/prerequisites/)

### Setup

```bash
# Clone the repository
git clone https://github.com/linkyourbin/nlbn_gui.git
cd nlbn_gui

# Install dependencies
npm install
```

### Development Mode

```bash
# Run in development mode with hot-reload
npm run tauri dev
```

### Build

```bash
# Build production bundles
npm run tauri build
```

This will generate:
- MSI installer in `src-tauri/target/release/bundle/msi/`
- NSIS installer in `src-tauri/target/release/bundle/nsis/`
- Portable EXE in `src-tauri/target/release/`

### Troubleshooting Build Issues

If you encounter network timeouts when downloading NSIS tools during build:

1. Download these files from the [latest release](https://github.com/linkyourbin/nlbn_gui/releases/latest):
   - `nsis-3.11.zip`
   - `nsis_tauri_utils.dll`

2. Copy `nsis_tauri_utils.dll` to:
   ```
   C:\Users\<YourUsername>\AppData\Local\tauri\NSIS\Plugins\x86-unicode\nsis_tauri_utils.dll
   ```

3. Run `npm run tauri build` again

## 🏗️ Technology Stack

### Frontend
- **TypeScript** - Type-safe JavaScript
- **Vite** - Fast build tool and dev server
- **HTML/CSS** - Modern responsive UI

### Backend
- **Rust** - Systems programming language for performance
- **Tauri 2.10.2** - Cross-platform desktop app framework
- **Tokio** - Async runtime
- **Reqwest** - HTTP client for LCSC API
- **Serde** - Serialization/deserialization
- **SQLite** - Local history database

### Conversion Engine
- **EasyEDA JSON Parser** - Parse component data from LCSC/EasyEDA API
- **KiCad Format Exporter** - Generate KiCad 6+ compatible files
  - `.kicad_sym` - Symbol library files
  - `.kicad_mod` - Footprint files
  - `.wrl` / `.step` - 3D model files

## 📁 Project Structure

```
nlbn_gui/
├── src/                          # Frontend source
│   ├── main.ts                   # Main TypeScript entry
│   ├── app-styles.css            # Application styles
│   └── assets/                   # Static assets
│       ├── nlbn.svg              # Banner logo
│       └── nlbn_simplified.png   # App icon
├── src-tauri/                    # Tauri backend
│   ├── src/
│   │   ├── commands.rs           # Tauri command handlers
│   │   ├── converter_impl.rs     # Conversion implementation
│   │   ├── state.rs              # Application state
│   │   ├── history.rs            # History database
│   │   ├── nlbn/                 # NLBN core modules
│   │   │   ├── easyeda/          # EasyEDA parser
│   │   │   ├── kicad/            # KiCad exporter
│   │   │   └── converter.rs      # Conversion logic
│   │   └── lib.rs                # Tauri entry point
│   ├── tauri.conf.json           # Tauri configuration
│   └── Cargo.toml                # Rust dependencies
├── public/                       # Public assets
│   └── favicon.png               # App favicon
├── index.html                    # HTML entry
├── package.json                  # Node dependencies
└── README.md                     # This file
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app) - For the amazing cross-platform framework
- [KiCad](https://www.kicad.org) - For the open-source EDA software
- [LCSC](https://lcsc.com) - For the component database and API
- [EasyEDA](https://easyeda.com) - For the component data format

## 📮 Contact

- GitHub: [@linkyourbin](https://github.com/linkyourbin)
- Issues: [GitHub Issues](https://github.com/linkyourbin/nlbn_gui/issues)

---

## 中文说明

NLBN GUI 是一个快速、直观的桌面应用程序，用于将 EasyEDA/LCSC 元件转换为 KiCad 格式。

### 主要特性

- **单个/批量转换** - 支持单个元件转换或批量处理多个 LCSC ID
- **实时进度跟踪** - 可视化进度更新，显示当前转换状态（n/m 格式）
- **文件导入** - 从文本文件导入 LCSC ID（支持逗号、空格和换行分隔）
- **转换历史** - 可滚动的历史记录，显示最近的转换和时间戳
- **智能错误报告** - 仅显示失败的元件，成功的自动添加到历史记录
- **灵活的输出选项** - 选择转换内容：符号、封装、3D 模型
- **简洁的界面设计** - 紧凑直观的界面，最佳窗口尺寸 1130x530
- **原生性能** - 使用 Rust 后端实现快速转换

### 下载安装

从 [Releases](https://github.com/linkyourbin/nlbn_gui/releases/latest) 下载最新版本：

| 安装包类型 | 大小 | 说明 |
|-----------|------|------|
| **NSIS 安装器**（推荐） | 4.6 MB | 带安装向导和开始菜单快捷方式 |
| **MSI 安装器** | 6.5 MB | Windows 标准安装包 |
| **便携版 EXE** | 18 MB | 无需安装直接运行 |

**系统要求：**
- Windows 10/11 (x64)
- 互联网连接（用于从 LCSC 获取元件数据）

### 快速开始

1. 下载并安装 NLBN GUI
2. 启动应用程序
3. 输入 LCSC ID（例如：`C5149201` 对应 STM32G431CBU6）
4. 选择输出目录
5. 选择转换选项（符号/封装/3D 模型）
6. 点击"Convert"或使用批量模式处理多个元件

### 技术栈

- **前端**: TypeScript + Vite
- **后端**: Rust + Tauri 2.10.2
- **数据库**: SQLite（历史记录）

### 开发指南

```bash
# 克隆仓库
git clone https://github.com/linkyourbin/nlbn_gui.git
cd nlbn_gui

# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build
```

### 许可证

MIT License

---

<div align="center">

🤖 使用 Tauri 和 Rust 精心打造

</div>
