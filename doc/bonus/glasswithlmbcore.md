# 黑色车漆 / 复合材料 (GlassWithBlackCore)

## 概述

通过新增 `GlassWithBlackCore` 材质，模拟现实世界中「透明涂层 (Clear Coat) + 深色底漆 (Base Coat)」的多层汽车烤漆效果。材质在**外表面**表现为标准玻璃的折射与菲涅尔反射，在**内表面**则表现为黑色朗伯体的漫反射吸收，从而获得深邃黑色 + 清晰镜面高光的视觉特征。

## 物理原理与多层结构

真实汽车漆由至少两层构成：

- **清漆层**：透明高光树脂，折射率约为 1.5，产生菲涅尔反射高光。
- **色漆层**：深色颜料颗粒，对穿透进来的光线进行漫反射（几乎全吸收）。

`GlassWithBlackCore` 在一个材质内部实现了这两层逻辑的切换：

| 击中面 | 物理等效 | 行为 |
|--------|----------|------|
| 外表面 (`front_face = true`) | 清漆层 | 折射 + 菲涅尔反射，衰减为白色 `(1, 1, 1)` |
| 内表面 (`front_face = false`) | 色漆层 | 黑色朗伯体漫反射，衰减为 `core_color` |

光线进入玻璃后在内表面按随机方向散射（类似 `Lambertian`），并被内核颜色强烈衰减。折射进入的光线几乎不出射，物体整体呈现深邃黑色，而表面则保留镜面反射（菲涅尔高光）。

## 实现原理

### 材质结构

```rust
pub struct GlassWithBlackCore {
    refraction_index: f64,  // 玻璃折射率（典型值 1.5）
    core_color: Color,       // 内核漫反射颜色（默认纯黑）
}
```

### `scatter` 方法逻辑

```rust
fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool {
    if rec.front_face {
        // 外表面：与 Dielecric 完全相同的折射/菲涅尔反射
        *attenuation = Color::new(1.0, 1.0, 1.0);
        // ... Schlick 近似 + 折射/反射 ...
    } else {
        // 内表面：黑色朗伯体漫反射
        *attenuation = self.core_color;
        let scatter_direction = rec.normal + random_unit_vector();
        // ... 零向量回退 ...
        *scattered = Ray::new_with_time(rec.p, scatter_direction, r_in.time());
        true
    }
}
```

外表面逻辑复用 `Dielecric` 的折射 + 菲涅尔反射代码（包括 Schlick 近似计算全内反射与能量分配）。内表面则与 `Lambertian` 散射逻辑一致，但衰减使用 `core_color` 而非纹理采样。

### 参数说明

| 参数 | 推荐范围 | 说明 |
|------|----------|------|
| `refraction_index` | 1.3 ~ 1.6 | 玻璃折射率。典型值 1.5 |
| `core_color` | `(0,0,0)` ~ `(0.1,0.1,0.1)` | 内核漫反射颜色。纯黑 `(0,0,0)` 最深邃；极暗灰如 `(0.05,0.05,0.05)` 更有立体感 |

## 使用方法

```rust
use crate::tools::material::GlassWithBlackCore;

// 纯黑内核（吸收所有折射光线）
let black_paint = Arc::new(GlassWithBlackCore::new(1.5));

// 暗灰内核（微弱散射，更有层次）
let black_paint_soft = Arc::new(GlassWithBlackCore::new_with_core(
    1.5,
    Color::new(0.05, 0.05, 0.05),
));
```

## 与双层球体方案 (Dielecric + Lambertian) 的对比

| 特性 | 双层球体方案 | `GlassWithBlackCore` |
|------|-------------|---------------------|
| 实现方式 | 两个同心球体（外玻璃 + 内朗伯） | 单一材质切换内外行为 |
| 额外几何体 | 需要内层球体与玻璃紧密嵌套 | 无额外几何体 |
| 内核击中点 | 内层球体表面（法线向内） | 玻璃内表面（法线仍向外） |
| 散射方向 | Lambertian 按内球法线散射 | Lambertian 按玻璃内表面法线散射 |
| 配置复杂度 | 多个球体协调位置/半径 | 单一材质，即插即用 |

## 应用场景

`src/work/black_car_paint.rs` 中已应用此材质：
- 四个对比球体：纯黑车漆、暗灰车漆、灰色朗伯体、金属铬球
- 顶部面光源 + 渐变背景
- 纯黑和暗灰车漆球均呈现深邃黑色加镜面高光

## 文件位置

- 材质定义：`src/tools/material.rs` → `GlassWithBlackCore`
- 场景应用：`src/work/black_car_paint.rs`