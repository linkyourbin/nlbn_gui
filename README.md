# NLBN - Neat Library Bring Now

Fast EasyEDA/LCSC to KiCad Converter with GUI

## 项目说明

这是从 nlbn_gui 移植到 Tauri 2 + TypeScript 的 NLBN 应用程序。

### 技术栈

- **前端**: TypeScript + Vite + HTML/CSS
- **后端**: Rust + Tauri 2
- **数据库**: SQLite (历史记录)

### 功能特性

- ✅ 单个组件转换 (LCSC ID)
- ✅ 自定义输出目录选择
- ✅ 转换选项配置 (Symbol/Footprint/3D Model)
- ✅ 转换历史记录
- ✅ 响应式 UI 设计
- ⏳ 批量转换 (TODO)
- ⏳ 实际的 NLBN 转换逻辑 (TODO)

## 开发

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

## 目录结构

```
nlbn_new/
├── src/                    # 前端源码
│   ├── main.ts            # 主 TypeScript 文件
│   ├── app-styles.css     # 应用样式
│   └── assets/            # 静态资源
│       └── nlbn.svg       # NLBN logo
├── src-tauri/             # Tauri 后端
│   └── src/
│       ├── commands.rs    # Tauri 命令
│       ├── types.rs       # 类型定义
│       ├── state.rs       # 应用状态
│       ├── history.rs     # 历史记录管理
│       ├── nlbn/          # NLBN 核心转换模块
│       └── lib.rs         # Tauri 入口
└── index.html             # HTML 入口文件
```

## 移植完成项目清单

- [x] 复制 Tauri 后端代码模块
- [x] 复制 nlbn 核心转换模块
- [x] 更新 Tauri 配置和依赖
- [x] 将 Dioxus 前端转换为 TypeScript/HTML/CSS
- [x] 复制资源文件和样式
- [x] 测试编译和构建

## TODO

- [ ] 实现实际的 NLBN 转换逻辑集成
- [ ] 添加批量转换功能
- [ ] 添加进度显示
- [ ] 完善错误处理
- [ ] 添加更多配置选项

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT
