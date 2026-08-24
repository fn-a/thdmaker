# thdslicer

[English](README.md) | 中文

3D 打印切片逻辑库：将 3D 网格切片为层，生成填充路径与 G-code。

## 模块结构

- `geometry`：二维几何（点、线段等）基础类型
- `layer`：切片层的轮廓构建与处理
- `infill`：填充图案（密集/稀疏填充）
- `indicate`：输出指示器抽象，支持生成 Bevy 网格或 G-code

切片输入为 Bevy 的 `Mesh` 与 `Transform`。