# NLBN GUI v1.0.0

NLBN (Neat Library Bring Now) - A desktop application for converting EasyEDA/LCSC components to KiCad format.

## Features

- 🔄 Batch conversion of LCSC components
- 📊 Real-time progress tracking with n/m format display
- 📁 Import LCSC IDs from text files (supports comma, space, newline delimiters)
- 📝 Conversion history with scrollable view (shows ~2 recent records)
- ✅ Smart result display (only shows failed components)
- 🎨 Clean and intuitive user interface with compact design

## Installation

Choose one of the following installers:

### Recommended
- **NSIS Installer** (4.6MB) - `NLBN_1.0.0_x64-setup.exe`
  - Windows installer with setup wizard
  - Smallest download size
  - Creates start menu shortcuts

### Alternative Options
- **MSI Installer** (6.5MB) - `NLBN_1.0.0_x64_en-US.msi`
  - Windows standard installer package
  - Good for enterprise deployment

- **Portable EXE** (18MB) - `nlbn_new.exe`
  - No installation required, just run
  - Perfect for USB drives or testing

## For Developers

If you encounter network issues when building from source, download the build tools:

1. Download `nsis-3.11.zip` and `nsis_tauri_utils.dll` from this release
2. Place both files in the project root directory
3. Copy `nsis_tauri_utils.dll` to:
   ```
   C:\Users\<YourUsername>\AppData\Local\tauri\NSIS\Plugins\x86-unicode\nsis_tauri_utils.dll
   ```
4. Run `npm run tauri build`

## System Requirements

- Windows 10/11 (x64)
- No additional dependencies required
- Internet connection (for fetching component data from LCSC)

## What's New in v1.0.0

- Initial release with complete feature set
- Real-time batch conversion progress updates
- File import functionality for bulk operations
- Optimized UI layout (1130x530 window size)
- History tracking with scrollable container
- Smart error reporting (failed components only)

## Technical Details

- Built with [Tauri](https://tauri.app) 2.10.2
- Backend: Rust (async/await, reqwest)
- Frontend: TypeScript + Vite
- Converts EasyEDA JSON to KiCad 6+ format

---

🤖 Built with ❤️ using Tauri and Rust
