# Altium Designer 支持实现指南

本文档详细说明如何为 NLBN GUI 添加 Altium Designer 文件格式支持。

## 📋 目录

1. [文件格式概述](#文件格式概述)
2. [实现步骤](#实现步骤)
3. [代码结构](#代码结构)
4. [关键代码示例](#关键代码示例)
5. [测试方案](#测试方案)

## 文件格式概述

### 1. SchLib (原理图库)

Altium Designer 的原理图库文件使用管道分隔的键值对格式：

```
|RECORD=1          # 记录类型：1 = 组件定义
|LIBREF=STM32      # 组件名称
|PARTCOUNT=1       # 零件数量
|DISPLAYMODECOUNT=1

|RECORD=2          # 记录类型：2 = 矩形
|OWNERINDEX=1      # 所属组件索引
|LOCATION.X=0      # X 坐标 (单位: mil)
|LOCATION.Y=0      # Y 坐标
|XSIZE=200         # 宽度
|YSIZE=400         # 高度
|COLOR=8388608     # 颜色 (RGB)
|ISSOLID=T         # 是否填充

|RECORD=41         # 记录类型：41 = 引脚
|OWNERINDEX=1
|LOCATION.X=0
|LOCATION.Y=100
|PINLENGTH=100     # 引脚长度
|ELECTRICAL=0      # 电气类型：0=Input, 1=I/O, 2=Output, 3=Open Collector, etc.
|PINCONGLOMERATE=1 # 引脚连接点位置
|NAME=PA0          # 引脚名称
|DESIGNATOR=1      # 引脚编号
```

### 2. PcbLib (PCB 封装库)

PCB 封装库的格式类似：

```
|RECORD=2          # 记录类型：2 = 封装定义
|NAME=QFN-48       # 封装名称
|DESCRIPTION=7x7mm QFN-48

|RECORD=3          # 记录类型：3 = 焊盘
|OWNERINDEX=0
|LAYER=TOP         # 层：TOP, BOTTOM, MULTILAYER
|X=100MIL          # X 坐标
|Y=100MIL          # Y 坐标
|XSIZE=15MIL       # X 尺寸
|YSIZE=60MIL       # Y 尺寸
|HOLESIZE=0MIL     # 孔径 (0表示 SMD)
|SHAPE=1           # 形状：0=Round, 1=Rectangle, 2=Octagonal
|PADMODE=0         # 模式：0=Simple, 1=Top-Middle-Bottom
|PLATED=T          # 是否镀铜
|NAME=1            # 焊盘编号

|RECORD=6          # 记录类型：6 = 线段
|LAYER=TOPOVERLAY  # 丝印层
|START.X=0MIL
|START.Y=0MIL
|END.X=100MIL
|END.Y=0MIL
|WIDTH=5MIL        # 线宽

|RECORD=16         # 记录类型：16 = 3D 模型
|OWNERINDEX=0
|MODELNAME=component.step
|MODELID={12345678-1234-1234-1234-123456789012}
|ROTATION.X=0
|ROTATION.Y=0
|ROTATION.Z=0
|Z=0MIL
```

## 实现步骤

### Step 1: 创建 Altium 模块结构

```bash
mkdir -p src-tauri/src/nlbn/altium
cd src-tauri/src/nlbn/altium
```

创建以下文件：
- `mod.rs` - 模块入口
- `symbol.rs` - 原理图符号数据结构
- `footprint.rs` - PCB 封装数据结构
- `symbol_exporter.rs` - SchLib 导出器
- `footprint_exporter.rs` - PcbLib 导出器
- `formatter.rs` - Altium 格式化工具

### Step 2: 定义数据结构

#### symbol.rs - 原理图符号

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdSymbol {
    pub libref: String,           // 组件名称
    pub description: String,      // 描述
    pub pins: Vec<AdPin>,         // 引脚列表
    pub rectangles: Vec<AdRectangle>,  // 矩形
    pub lines: Vec<AdLine>,       // 线段
    pub texts: Vec<AdText>,       // 文本
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdPin {
    pub x: i32,                   // X 坐标 (mil)
    pub y: i32,                   // Y 坐标 (mil)
    pub length: i32,              // 引脚长度 (mil)
    pub name: String,             // 引脚名称
    pub designator: String,       // 引脚编号
    pub electrical: PinElectrical, // 电气类型
    pub orientation: PinOrientation, // 方向
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinElectrical {
    Input,           // 输入
    IO,              // 输入/输出
    Output,          // 输出
    OpenCollector,   // 开集
    Passive,         // 被动
    HiZ,             // 高阻
    OpenEmitter,     // 开射
    Power,           // 电源
}

impl PinElectrical {
    pub fn to_altium_code(&self) -> i32 {
        match self {
            PinElectrical::Input => 0,
            PinElectrical::IO => 1,
            PinElectrical::Output => 2,
            PinElectrical::OpenCollector => 3,
            PinElectrical::Passive => 4,
            PinElectrical::HiZ => 5,
            PinElectrical::OpenEmitter => 6,
            PinElectrical::Power => 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinOrientation {
    Right,   // 引脚向右
    Left,    // 引脚向左
    Up,      // 引脚向上
    Down,    // 引脚向下
}

impl PinOrientation {
    pub fn to_altium_code(&self) -> i32 {
        match self {
            PinOrientation::Right => 0,
            PinOrientation::Left => 2,
            PinOrientation::Up => 1,
            PinOrientation::Down => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdRectangle {
    pub x: i32,       // X 坐标 (mil)
    pub y: i32,       // Y 坐标 (mil)
    pub width: i32,   // 宽度 (mil)
    pub height: i32,  // 高度 (mil)
    pub color: u32,   // 颜色 (RGB)
    pub is_solid: bool, // 是否填充
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdLine {
    pub start_x: i32,  // 起点 X (mil)
    pub start_y: i32,  // 起点 Y (mil)
    pub end_x: i32,    // 终点 X (mil)
    pub end_y: i32,    // 终点 Y (mil)
    pub width: i32,    // 线宽 (mil)
    pub color: u32,    // 颜色 (RGB)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdText {
    pub x: i32,        // X 坐标 (mil)
    pub y: i32,        // Y 坐标 (mil)
    pub text: String,  // 文本内容
    pub height: i32,   // 字体高度 (mil)
    pub rotation: f64, // 旋转角度 (度)
}
```

#### footprint.rs - PCB 封装

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdFootprint {
    pub name: String,             // 封装名称
    pub description: String,      // 描述
    pub pads: Vec<AdPad>,         // 焊盘列表
    pub lines: Vec<AdLine>,       // 线段
    pub arcs: Vec<AdArc>,         // 圆弧
    pub texts: Vec<AdText>,       // 文本
    pub model_3d: Option<Ad3DModel>, // 3D 模型
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdPad {
    pub x: f64,           // X 坐标 (mil)
    pub y: f64,           // Y 坐标 (mil)
    pub width: f64,       // 宽度 (mil)
    pub height: f64,      // 高度 (mil)
    pub hole_size: f64,   // 孔径 (mil, 0表示SMD)
    pub shape: PadShape,  // 形状
    pub name: String,     // 焊盘编号
    pub layer: PadLayer,  // 层
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PadShape {
    Round,      // 圆形
    Rectangle,  // 矩形
    Octagonal,  // 八角形
    RoundRect,  // 圆角矩形
}

impl PadShape {
    pub fn to_altium_code(&self) -> i32 {
        match self {
            PadShape::Round => 0,
            PadShape::Rectangle => 1,
            PadShape::Octagonal => 2,
            PadShape::RoundRect => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PadLayer {
    Top,         // 顶层
    Bottom,      // 底层
    MultiLayer,  // 多层 (通孔)
}

impl PadLayer {
    pub fn to_altium_name(&self) -> &'static str {
        match self {
            PadLayer::Top => "TOP",
            PadLayer::Bottom => "BOTTOM",
            PadLayer::MultiLayer => "MULTILAYER",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdLine {
    pub start_x: f64,  // 起点 X (mil)
    pub start_y: f64,  // 起点 Y (mil)
    pub end_x: f64,    // 终点 X (mil)
    pub end_y: f64,    // 终点 Y (mil)
    pub width: f64,    // 线宽 (mil)
    pub layer: String, // 层名 (TOPOVERLAY, BOTTOMOVERLAY, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdArc {
    pub center_x: f64, // 圆心 X (mil)
    pub center_y: f64, // 圆心 Y (mil)
    pub radius: f64,   // 半径 (mil)
    pub start_angle: f64, // 起始角度 (度)
    pub end_angle: f64,   // 结束角度 (度)
    pub width: f64,    // 线宽 (mil)
    pub layer: String, // 层名
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdText {
    pub x: f64,        // X 坐标 (mil)
    pub y: f64,        // Y 坐标 (mil)
    pub text: String,  // 文本内容
    pub height: f64,   // 字体高度 (mil)
    pub width: f64,    // 字体宽度 (mil)
    pub rotation: f64, // 旋转角度 (度)
    pub layer: String, // 层名
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ad3DModel {
    pub filename: String,  // 模型文件名 (e.g., "component.step")
    pub rotation_x: f64,   // X 轴旋转 (度)
    pub rotation_y: f64,   // Y 轴旋转 (度)
    pub rotation_z: f64,   // Z 轴旋转 (度)
    pub offset_z: f64,     // Z 轴偏移 (mil)
}
```

### Step 3: 实现导出器

#### symbol_exporter.rs - 原理图库导出器

```rust
use std::fs::File;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;
use super::symbol::*;

pub struct SymbolExporter;

impl SymbolExporter {
    pub fn new() -> Self {
        Self
    }

    /// 导出原理图符号为 SchLib 文件
    pub fn export(&self, symbol: &AdSymbol, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();

        // 1. 写入文件头
        self.write_header(&mut content);

        // 2. 写入组件定义
        self.write_component(&mut content, symbol);

        // 3. 写入图形元素
        self.write_rectangles(&mut content, &symbol.rectangles);
        self.write_lines(&mut content, &symbol.lines);
        self.write_texts(&mut content, &symbol.texts);

        // 4. 写入引脚
        self.write_pins(&mut content, &symbol.pins);

        // 5. 保存文件
        let mut file = File::create(output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    fn write_header(&self, content: &mut String) {
        content.push_str("|HEADER=Protel for Windows - Schematic Library Editor Binary File Version 5.0\n");
        content.push_str("|WEIGHT=748\n");
        content.push_str("|MINORVERSION=2\n");
        content.push_str("|USEMBCS=T\n");
        content.push_str("\n");
    }

    fn write_component(&self, content: &mut String, symbol: &AdSymbol) {
        content.push_str("|RECORD=1\n");
        content.push_str(&format!("|LIBREF={}\n", symbol.libref));
        content.push_str(&format!("|COMPONENTDESCRIPTION={}\n", symbol.description));
        content.push_str("|PARTCOUNT=1\n");
        content.push_str("|DISPLAYMODECOUNT=1\n");
        content.push_str("|INDEXINSHEET=-1\n");
        content.push_str("|OWNERPARTID=-1\n");
        content.push_str("|LOCATION.X=0\n");
        content.push_str("|LOCATION.Y=0\n");
        content.push_str("|LIBRARYPATH=*\n");
        content.push_str("|SOURCELIBRARYNAME=*\n");
        content.push_str("|TARGETFILENAME=*\n");
        content.push_str("\n");
    }

    fn write_rectangles(&self, content: &mut String, rectangles: &[AdRectangle]) {
        for (index, rect) in rectangles.iter().enumerate() {
            content.push_str("|RECORD=2\n");
            content.push_str(&format!("|OWNERINDEX={}\n", index + 1));
            content.push_str(&format!("|LOCATION.X={}\n", rect.x));
            content.push_str(&format!("|LOCATION.Y={}\n", rect.y));
            content.push_str(&format!("|XSIZE={}\n", rect.width));
            content.push_str(&format!("|YSIZE={}\n", rect.height));
            content.push_str(&format!("|COLOR={}\n", rect.color));
            content.push_str("|AREACOLOR=16777215\n");
            content.push_str(&format!("|ISSOLID={}\n", if rect.is_solid { "T" } else { "F" }));
            content.push_str("\n");
        }
    }

    fn write_lines(&self, content: &mut String, lines: &[AdLine]) {
        for line in lines {
            content.push_str("|RECORD=13\n");
            content.push_str("|OWNERINDEX=1\n");
            content.push_str("|OWNERPARTID=-1\n");
            content.push_str("|LINEWIDTH=1\n");
            content.push_str(&format!("|COLOR={}\n", line.color));
            content.push_str(&format("|LOCATIONCOUNT=2\n"));
            content.push_str(&format!("|X1={}\n", line.start_x));
            content.push_str(&format!("|Y1={}\n", line.start_y));
            content.push_str(&format!("|X2={}\n", line.end_x));
            content.push_str(&format!("|Y2={}\n", line.end_y));
            content.push_str("\n");
        }
    }

    fn write_texts(&self, content: &mut String, texts: &[AdText]) {
        for text in texts {
            content.push_str("|RECORD=4\n");
            content.push_str("|OWNERINDEX=1\n");
            content.push_str(&format!("|LOCATION.X={}\n", text.x));
            content.push_str(&format!("|LOCATION.Y={}\n", text.y));
            content.push_str(&format!("|TEXT={}\n", text.text));
            content.push_str(&format!("|FONTID=1\n"));
            content.push_str("|COLOR=0\n");
            content.push_str("\n");
        }
    }

    fn write_pins(&self, content: &mut String, pins: &[AdPin]) {
        for pin in pins {
            content.push_str("|RECORD=41\n");
            content.push_str("|OWNERINDEX=1\n");
            content.push_str("|OWNERPARTID=-1\n");
            content.push_str(&format!("|LOCATION.X={}\n", pin.x));
            content.push_str(&format!("|LOCATION.Y={}\n", pin.y));
            content.push_str(&format!("|PINLENGTH={}\n", pin.length));
            content.push_str(&format!("|ELECTRICAL={}\n", pin.electrical.to_altium_code()));
            content.push_str(&format!("|PINCONGLOMERATE={}\n", pin.orientation.to_altium_code()));
            content.push_str(&format!("|NAME={}\n", pin.name));
            content.push_str(&format!("|DESIGNATOR={}\n", pin.designator));
            content.push_str("|COLOR=0\n");
            content.push_str("\n");
        }
    }
}
```

#### footprint_exporter.rs - PCB 封装库导出器

```rust
use std::fs::File;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;
use super::footprint::*;

pub struct FootprintExporter;

impl FootprintExporter {
    pub fn new() -> Self {
        Self
    }

    /// 导出 PCB 封装为 PcbLib 文件
    pub fn export(&self, footprint: &AdFootprint, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();

        // 1. 写入文件头
        self.write_header(&mut content);

        // 2. 写入封装定义
        self.write_footprint_def(&mut content, footprint);

        // 3. 写入焊盘
        self.write_pads(&mut content, &footprint.pads);

        // 4. 写入图形元素
        self.write_lines(&mut content, &footprint.lines);
        self.write_arcs(&mut content, &footprint.arcs);
        self.write_texts(&mut content, &footprint.texts);

        // 5. 写入 3D 模型
        if let Some(model) = &footprint.model_3d {
            self.write_3d_model(&mut content, model);
        }

        // 6. 保存文件
        let mut file = File::create(output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    fn write_header(&self, content: &mut String) {
        content.push_str("|HEADER=Protel for Windows - PCB Library Binary File Version 5.0\n");
        content.push_str("\n");
    }

    fn write_footprint_def(&self, content: &mut String, footprint: &AdFootprint) {
        content.push_str("|RECORD=2\n");
        content.push_str(&format!("|NAME={}\n", footprint.name));
        content.push_str(&format!("|DESCRIPTION={}\n", footprint.description));
        content.push_str("\n");
    }

    fn write_pads(&self, content: &mut String, pads: &[AdPad]) {
        for pad in pads {
            content.push_str("|RECORD=3\n");
            content.push_str("|OWNERINDEX=0\n");
            content.push_str(&format!("|LAYER={}\n", pad.layer.to_altium_name()));
            content.push_str(&format!("|X={}MIL\n", pad.x));
            content.push_str(&format!("|Y={}MIL\n", pad.y));
            content.push_str(&format!("|XSIZE={}MIL\n", pad.width));
            content.push_str(&format!("|YSIZE={}MIL\n", pad.height));
            content.push_str(&format!("|HOLESIZE={}MIL\n", pad.hole_size));
            content.push_str(&format!("|SHAPE={}\n", pad.shape.to_altium_code()));
            content.push_str("|PADMODE=0\n");
            content.push_str(&format!("|PLATED={}\n", if pad.hole_size > 0.0 { "T" } else { "F" }));
            content.push_str(&format!("|NAME={}\n", pad.name));
            content.push_str("\n");
        }
    }

    fn write_lines(&self, content: &mut String, lines: &[AdLine]) {
        for line in lines {
            content.push_str("|RECORD=6\n");
            content.push_str(&format!("|LAYER={}\n", line.layer));
            content.push_str(&format!("|START.X={}MIL\n", line.start_x));
            content.push_str(&format!("|START.Y={}MIL\n", line.start_y));
            content.push_str(&format!("|END.X={}MIL\n", line.end_x));
            content.push_str(&format!("|END.Y={}MIL\n", line.end_y));
            content.push_str(&format!("|WIDTH={}MIL\n", line.width));
            content.push_str("\n");
        }
    }

    fn write_arcs(&self, content: &mut String, arcs: &[AdArc]) {
        for arc in arcs {
            content.push_str("|RECORD=7\n");
            content.push_str(&format!("|LAYER={}\n", arc.layer));
            content.push_str(&format!("|LOCATION.X={}MIL\n", arc.center_x));
            content.push_str(&format!("|LOCATION.Y={}MIL\n", arc.center_y));
            content.push_str(&format!("|RADIUS={}MIL\n", arc.radius));
            content.push_str(&format!("|STARTANGLE={}\n", arc.start_angle));
            content.push_str(&format!("|ENDANGLE={}\n", arc.end_angle));
            content.push_str(&format!("|WIDTH={}MIL\n", arc.width));
            content.push_str("\n");
        }
    }

    fn write_texts(&self, content: &mut String, texts: &[AdText]) {
        for text in texts {
            content.push_str("|RECORD=8\n");
            content.push_str(&format!("|LAYER={}\n", text.layer));
            content.push_str(&format!("|X={}MIL\n", text.x));
            content.push_str(&format!("|Y={}MIL\n", text.y));
            content.push_str(&format!("|TEXT={}\n", text.text));
            content.push_str(&format!("|HEIGHT={}MIL\n", text.height));
            content.push_str(&format!("|WIDTH={}MIL\n", text.width));
            content.push_str(&format!("|ROTATION={}\n", text.rotation));
            content.push_str("\n");
        }
    }

    fn write_3d_model(&self, content: &mut String, model: &Ad3DModel) {
        let model_id = Uuid::new_v4();

        content.push_str("|RECORD=16\n");
        content.push_str("|OWNERINDEX=0\n");
        content.push_str(&format!("|MODELNAME={}\n", model.filename));
        content.push_str(&format!("|MODELID={{{}}}\n", model_id));
        content.push_str("|MODELDESCRIPTION=\n");
        content.push_str(&format!("|ROTATION.X={}\n", model.rotation_x));
        content.push_str(&format!("|ROTATION.Y={}\n", model.rotation_y));
        content.push_str(&format!("|ROTATION.Z={}\n", model.rotation_z));
        content.push_str(&format!("|Z={}MIL\n", model.offset_z));
        content.push_str("|CHECKSUM=\n");
        content.push_str("\n");
    }
}
```

### Step 4: 添加转换逻辑

创建从 EasyEDA 到 Altium 的转换器：

```rust
// src-tauri/src/nlbn/altium/converter.rs

use super::super::easyeda::models::*;
use super::symbol::*;
use super::footprint::*;

pub struct AltiumConverter;

impl AltiumConverter {
    pub fn new() -> Self {
        Self
    }

    /// 将 EasyEDA 符号转换为 Altium 符号
    pub fn convert_symbol(&self, ee_symbol: &EeSymbol) -> AdSymbol {
        let mut ad_symbol = AdSymbol {
            libref: ee_symbol.name.clone(),
            description: ee_symbol.description.clone(),
            pins: Vec::new(),
            rectangles: Vec::new(),
            lines: Vec::new(),
            texts: Vec::new(),
        };

        // 转换引脚
        for ee_pin in &ee_symbol.pins {
            ad_symbol.pins.push(self.convert_pin(ee_pin));
        }

        // 转换矩形（作为符号外框）
        for ee_rect in &ee_symbol.rectangles {
            ad_symbol.rectangles.push(self.convert_rectangle(ee_rect));
        }

        ad_symbol
    }

    /// 将 EasyEDA 封装转换为 Altium 封装
    pub fn convert_footprint(&self, ee_footprint: &EeFootprint) -> AdFootprint {
        let mut ad_footprint = AdFootprint {
            name: ee_footprint.name.clone(),
            description: format!("{}", ee_footprint.name),
            pads: Vec::new(),
            lines: Vec::new(),
            arcs: Vec::new(),
            texts: Vec::new(),
            model_3d: None,
        };

        // 转换焊盘
        for ee_pad in &ee_footprint.pads {
            ad_footprint.pads.push(self.convert_pad(ee_pad));
        }

        // 如果有 3D 模型，添加引用
        if ee_footprint.has_3d_model {
            ad_footprint.model_3d = Some(Ad3DModel {
                filename: format!("{}.step", ee_footprint.name),
                rotation_x: 0.0,
                rotation_y: 0.0,
                rotation_z: 0.0,
                offset_z: 0.0,
            });
        }

        ad_footprint
    }

    fn convert_pin(&self, ee_pin: &EePin) -> AdPin {
        // EasyEDA 使用 0.1 英寸网格，Altium 使用 mil
        // 1 inch = 1000 mil, 0.1 inch = 100 mil
        let x_mil = (ee_pin.x * 100.0) as i32;  // 转换为 mil
        let y_mil = (ee_pin.y * 100.0) as i32;

        AdPin {
            x: x_mil,
            y: y_mil,
            length: 100,  // 默认引脚长度 100 mil
            name: ee_pin.name.clone(),
            designator: ee_pin.number.clone(),
            electrical: PinElectrical::Passive,  // 默认为被动
            orientation: PinOrientation::Right,  // 默认向右
        }
    }

    fn convert_rectangle(&self, ee_rect: &EeRectangle) -> AdRectangle {
        let x_mil = (ee_rect.x * 100.0) as i32;
        let y_mil = (ee_rect.y * 100.0) as i32;
        let width_mil = (ee_rect.width * 100.0) as i32;
        let height_mil = (ee_rect.height * 100.0) as i32;

        AdRectangle {
            x: x_mil,
            y: y_mil,
            width: width_mil,
            height: height_mil,
            color: 0x000000,  // 黑色
            is_solid: false,
        }
    }

    fn convert_pad(&self, ee_pad: &EePad) -> AdPad {
        // EasyEDA 使用 mm，Altium 使用 mil
        // 1 mm = 39.3701 mil
        let x_mil = ee_pad.x * 39.3701;
        let y_mil = ee_pad.y * 39.3701;
        let width_mil = ee_pad.width * 39.3701;
        let height_mil = ee_pad.height * 39.3701;
        let hole_mil = ee_pad.hole_diameter.unwrap_or(0.0) * 39.3701;

        let shape = if ee_pad.shape == "OVAL" {
            PadShape::Round
        } else if ee_pad.shape == "RECT" {
            PadShape::Rectangle
        } else {
            PadShape::Round
        };

        let layer = if hole_mil > 0.0 {
            PadLayer::MultiLayer  // 通孔
        } else {
            PadLayer::Top  // SMD
        };

        AdPad {
            x: x_mil,
            y: y_mil,
            width: width_mil,
            height: height_mil,
            hole_size: hole_mil,
            shape,
            name: ee_pad.number.clone(),
            layer,
        }
    }
}
```

### Step 5: 更新 UI

在 `index.html` 中添加目标格式选择：

```html
<div class="option-group">
    <label class="checkbox-label">
        <input type="checkbox" id="target-kicad" checked>
        <span>KiCad</span>
    </label>
    <label class="checkbox-label">
        <input type="checkbox" id="target-altium">
        <span>Altium Designer</span>
    </label>
</div>
```

在 `main.ts` 中处理目标格式选择：

```typescript
interface ConvertOptions {
    includeSymbol: boolean;
    includeFootprint: boolean;
    include3DModel: boolean;
    targetKicad: boolean;      // 新增
    targetAltium: boolean;     // 新增
}

function getConvertOptions(): ConvertOptions {
    return {
        includeSymbol: (document.getElementById('include-symbol') as HTMLInputElement).checked,
        includeFootprint: (document.getElementById('include-footprint') as HTMLInputElement).checked,
        include3DModel: (document.getElementById('include-3d') as HTMLInputElement).checked,
        targetKicad: (document.getElementById('target-kicad') as HTMLInputElement).checked,
        targetAltium: (document.getElementById('target-altium') as HTMLInputElement).checked,
    };
}
```

### Step 6: 更新后端命令

在 `commands.rs` 中添加 Altium 导出支持：

```rust
use crate::nlbn::altium::{converter::AltiumConverter, symbol_exporter::SymbolExporter, footprint_exporter::FootprintExporter};

#[tauri::command]
pub async fn convert_component(
    lcsc_id: String,
    output_dir: String,
    options: ConvertOptions,
) -> Result<ConversionResult, String> {
    // ... 获取 EasyEDA 数据 ...

    let mut result = ConversionResult {
        success: true,
        message: String::new(),
        symbol_path: None,
        footprint_path: None,
        model_path: None,
    };

    // KiCad 导出 (现有逻辑)
    if options.target_kicad {
        // ... 现有的 KiCad 导出代码 ...
    }

    // Altium Designer 导出 (新增)
    if options.target_altium {
        let altium_converter = AltiumConverter::new();

        // 导出原理图符号
        if options.include_symbol && ee_symbol.is_some() {
            let ad_symbol = altium_converter.convert_symbol(&ee_symbol.unwrap());
            let symbol_path = Path::new(&output_dir)
                .join("altium")
                .join(format!("{}.SchLib", component_name));

            let symbol_exporter = SymbolExporter::new();
            symbol_exporter.export(&ad_symbol, &symbol_path)
                .map_err(|e| format!("Altium symbol export failed: {}", e))?;
        }

        // 导出 PCB 封装
        if options.include_footprint && ee_footprint.is_some() {
            let ad_footprint = altium_converter.convert_footprint(&ee_footprint.unwrap());
            let footprint_path = Path::new(&output_dir)
                .join("altium")
                .join(format!("{}.PcbLib", component_name));

            let footprint_exporter = FootprintExporter::new();
            footprint_exporter.export(&ad_footprint, &footprint_path)
                .map_err(|e| format!("Altium footprint export failed: {}", e))?;
        }

        // 复制 3D 模型 (STEP 文件通用)
        if options.include_3d_model && step_data.is_some() {
            let model_src = kicad_model_path;  // 复用 KiCad 的 STEP 文件
            let model_dst = Path::new(&output_dir)
                .join("altium")
                .join("3d")
                .join(format!("{}.step", component_name));

            std::fs::create_dir_all(model_dst.parent().unwrap())?;
            std::fs::copy(&model_src, &model_dst)?;
        }
    }

    Ok(result)
}
```

## 测试方案

### 测试步骤

1. **单元测试** - 测试各个转换函数
2. **集成测试** - 测试完整转换流程
3. **实际验证** - 在 Altium Designer 中导入生成的文件

### 验证清单

- [ ] SchLib 文件可以在 Altium Designer 中打开
- [ ] 符号引脚位置正确
- [ ] 引脚编号和名称正确
- [ ] PcbLib 文件可以在 Altium Designer 中打开
- [ ] 焊盘位置和尺寸正确
- [ ] 焊盘编号正确
- [ ] 3D 模型可以正确显示
- [ ] 3D 模型位置和方向正确

## 坐标系统转换

### EasyEDA 到 Altium 的单位转换

| 类型 | EasyEDA | Altium | 转换公式 |
|-----|---------|--------|---------|
| 原理图 | 0.1 inch | mil (0.001 inch) | `altium_mil = easyeda_unit * 100` |
| PCB | mm | mil | `altium_mil = easyeda_mm * 39.3701` |
| 角度 | 度 | 度 | 直接使用 |

### 坐标原点

- **EasyEDA**: 左上角为原点
- **Altium**: 中心为原点
- **转换**: 需要计算边界框并居中

## 常见问题

### Q1: Altium Designer 打不开生成的文件？

**A**: 检查文件格式版本，确保使用文本格式而非二进制格式。

### Q2: 引脚位置不对？

**A**: 检查单位转换和坐标系统转换是否正确。

### Q3: 3D 模型不显示？

**A**: 检查 STEP 文件路径、模型文件是否存在、模型 ID 是否正确。

### Q4: 焊盘形状不对？

**A**: Altium 支持的形状有限，某些 EasyEDA 形状需要近似转换。

## 参考资源

- [Altium Designer File Format Documentation](https://www.altium.com/documentation/)
- [STEP File Format Specification](https://en.wikipedia.org/wiki/ISO_10303-21)
- [PCB Design Guidelines](https://www.altium.com/documentation/altium-designer/pcb-design-guidelines)

## 下一步计划

1. ✅ 完成基础数据结构定义
2. ✅ 实现 SchLib 导出器
3. ✅实现 PcbLib 导出器
4. ⏳ 添加单位转换测试
5. ⏳ 在实际项目中验证
6. ⏳ 优化转换精度
7. ⏳ 添加更多图形元素支持

---

📝 本文档会持续更新，欢迎提出改进建议！
