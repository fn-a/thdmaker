# thdfiler

[English](README.md) | 中文

3D 制造文件格式库：支持 AMF、STL、3MF、GCode 等格式的读取与写入。

## 模块结构

- `tmf`：3MF 格式（含梁格、布尔运算、位移、材质、生产、安全内容、切片体扩展）
- `amf`：AMF 格式
- `stl`：STL 格式（二进制与 ASCII）
- `gcode`：GCode 格式
- `util`：路径等通用工具

每个格式模块均按 `define` / `reader` / `writer` / `error` 组织。

被 [`thdslicer`](../thdslicer/README.zh.md) 与 [`thdviewer`](../thdviewer/README.zh.md) 依赖。
