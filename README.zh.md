# THD Maker - 3D打印创作
一个基于Bevy的 3D 打印设计软件。

## 概述

THD Maker 是一个功能强大的应用，专为 3D 打印工作流设计，包括 3MF、AMF、STL 等文件读写、预览、切片和 GCode 生成。基于 Rust 和 Bevy 构建，确保安全性和高性能。

## 特性

- **多格式支持**：全面支持主流 3D 制造格式（3MF、AMF、STL）
- **高性能**：利用 Rust 的零成本抽象实现最佳性能
- **内存安全**：基于 Rust 的所有权模型构建，防止内存相关错误
- **可扩展设计**：模块化架构，便于轻松添加新格式
- **跨平台**：支持 Windows、macOS 和 Linux
- **3D 设计软件**：基于 Bevy 的 3D 打印设计应用，支持文件读写、预览、切片和 GCode 生成

## 文件格式参考

### 3MF (3D Manufacturing Format)
- [3MF 规范](https://3mf.io/spec/)
- [官方 3MF 联盟库](https://github.com/3MFConsortium/lib3mf)
- [国会图书馆格式描述](https://www.loc.gov/preservation/digital/formats/fdd/fdd000557.shtml)

### AMF (Additive Manufacturing File Format)
ISO-52915 标准
- [ASTM 国际](https://www.astm.org/)
- [ISO 标准](https://www.iso.org/standard/74640.html)
- [ISO TC261 委员会](https://committee.iso.org/home/tc261)
- [AMF XSD 架构](https://standards.iso.org/iso/52915/ISOASTM52915(2016)AMF.xsd)
- [维基百科：增材制造文件格式](https://en.wikipedia.org/wiki/Additive_manufacturing_file_format)

### STL (Stereolithography)
- [维基百科：STL 格式](https://en.wikipedia.org/wiki/STL_(file_format))
- [国会图书馆格式描述](https://www.loc.gov/preservation/digital/formats/fdd/fdd000505.shtml)

### G-code (Numerical Control Programming)
- [维基百科：G-code](https://en.wikipedia.org/wiki/G-code)
- [RepRap G-code](https://reprap.org/wiki/G-code)
- [Marlin G-code 参考](https://marlinfw.org/meta/gcode)
- [Klipper G-code 文档](https://www.klipper3d.org/G-Codes.html)
- [LinuxCNC G-code 概览](https://linuxcnc.org/docs/html/gcode/overview.html)
- [GRBL G-code 参考](https://github.com/gnea/grbl/wiki)
- [NIST RS274NGC](https://www.nist.gov/publications/nist-rs274ngc-interpreter-version-3)

### STEP/STP (Standard for the Exchange of Product Data)
ISO-10303 标准
- [ISO 标准概览](https://www.iso.org/standard/83105.html)
- [ISO 标准详情](https://www.iso.org/standard/63141.html)
- [STEP 零件列表](https://standards.iso.org/iso/10303/STEP_Parts_List.htm)
- [STEP 工具](https://www.steptools.com/)
- [STEP 标准文档](https://www.steptools.com/stds/step)
- [STEP 第 21 部分第 3 版](https://steptools.com/stds/step/IS_final_p21e3.html)
- [维基百科：ISO 10303](https://en.wikipedia.org/wiki/ISO_10303)
- [维基百科：ISO 10303-21](https://en.wikipedia.org/wiki/ISO_10303-21)
- [国会图书馆格式描述](https://www.loc.gov/preservation/digital/formats/fdd/fdd000448.shtml)
- [FreeCAD](https://www.freecad.org)
- [OpenCASCADE](https://www.opencascade.com)
- [OpenCASCADE 开发者门户](https://dev.opencascade.org)

### OBJ (Wavefront OBJ)
- [国会图书馆格式描述](https://www.loc.gov/preservation/digital/formats/fdd/fdd000507.shtml)
- [维基百科：Wavefront OBJ 文件](https://en.wikipedia.org/wiki/Wavefront_.obj_file)

## 安装

将以下内容添加到您的 `Cargo.toml` 中：

```bash
cargo add thdmaker
```

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件了解详细信息。

## 致谢

- 感谢所有维护 3D 文件格式规范的组织和标准机构
- 受其他语言现有库的启发
- 使用优秀的 Rust 生态系统构建

## 链接

- [文档](https://docs.rs/thdmaker)
- [Crates.io](https://crates.io/crates/thdmaker)
- [变更日志](CHANGELOG.md)

## 参考

### OBJ
- [Twinklebear/tobj](https://github.com/Twinklebear/tobj)

### STEP/STP
- [stepcode/stepcode](https://github.com/stepcode/stepcode)
- [mozman/steputils](https://github.com/mozman/steputils)
- [ricosjp/ruststep](https://github.com/ricosjp/ruststep)
- [FreeCAD/FreeCAD](https://github.com/FreeCAD/FreeCAD)

### GCODE等相关
- [Michael-F-Bryan/gcode-rs](https://github.com/Michael-F-Bryan/gcode-rs)
- [Slic3r/Slic3r](https://github.com/Slic3r/Slic3r)
- [Klipper3d/klipper](https://github.com/Klipper3d/klipper)
- [MarlinFirmware/Marlin](https://github.com/MarlinFirmware/Marlin)
