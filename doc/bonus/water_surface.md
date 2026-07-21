# 波纹水面 (Water Surface)

## 概述

通过新增 `WaterMetal` 材质，在理想镜面反射的基础上，用 **Perlin 噪声** 对反射方向进行空间连续的扰动，模拟水面波纹扭曲反射的效果。

## 实现原理

### `Metal` 的局限

传统 `Metal` 材质使用 `fuzz` 参数控制粗糙度，其扰动量由 `random_unit_vector()` 生成——每次采样的扰动方向完全随机，相邻像素的反射没有空间关联性。这对于磨砂金属是合适的，但无法模拟水面的**连续波纹**。

### `WaterMetal` 的改进

```rust
pub struct WaterMetal {
    albedo: Color,           // 反照率
    wave_scale: f64,         // 波纹空间频率
    wave_strength: f64,      // 波纹强度（类似 fuzz）
    noise: Perlin,           // Perlin 噪声实例
}
```

**核心思路**：

1. 计算理想反射方向：`reflect(r_in.direction(), rec.normal)`
2. 在击中点 `p` 处，用 Perlin 噪声在 XZ 平面采样两个独立噪声值，组合成一个**空间连续**的扰动向量
3. 将扰动叠加到反射方向，产生波纹扭曲

```rust
// 在 XZ 方向采样两个独立的 Perlin 噪声值
let nx = self.noise.noise(&(self.wave_scale * rec.p));
let nz = self.noise.noise(&(self.wave_scale * (rec.p + Vec3::new(7.31, 3.17, 0.0))));
let perturbation = Vec3::new(nx, 0.0, nz) * self.wave_strength;

let scattered_dir = unit_vector(reflected) + perturbation;
```

由于 Perlin 噪声**空间连续**，相邻击中点会获得相近的扰动，从而呈现自然的波纹图案。与 `Metal` 的纯随机 fuzz 根本不同。

### 参数说明

| 参数 | 推荐范围 | 说明 |
|------|----------|------|
| `wave_scale` | 5.0 ~ 20.0 | 波纹空间频率。值越大，波纹越密集 |
| `wave_strength` | 0.05 ~ 0.3 | 波纹强度。值越大，反射扭曲越剧烈 |

## 使用方法

```rust
use crate::tools::material::WaterMetal;

// 白色水面，波纹频率 8.0，强度 0.15
let water = Arc::new(WaterMetal::new(
    Color::new(1.0, 1.0, 1.0),
    8.0,   // wave_scale
    0.15,  // wave_strength
));
```

## 应用场景

`src/work/earth_night.rs` 中已应用此材质：
- 发光照片平面（NASA 全球夜光图）的倒影投射到水面
- 波纹扰动使倒影呈现自然的扭曲效果
- 纯黑背景下，水面反射是画面唯一的间接光照来源

## 与 `Metal` 的对比

| 特性 | `Metal` (fuzz) | `WaterMetal` |
|------|---------------|-------------|
| 扰动来源 | `random_unit_vector()` | Perlin 噪声 |
| 空间连续性 | 无（各像素独立随机） | 有（空间连续） |
| 视觉效果 | 磨砂/粗糙金属 | 水波纹 |
| 参数 | `albedo` + `fuzz` | `albedo` + `wave_scale` + `wave_strength` |

## 文件位置

- 材质定义：`src/tools/material.rs` → `WaterMetal`
- 场景应用：`src/work/earth_night.rs`