# 照片作为环境光源

## 概述

利用图像文件（PNG、JPG 等）的像素颜色直接作为场景中的光源，实现基于图像的环境照明效果。例如将 NASA 全球夜光图的灯光数据贴到发光平面上，让照片中的城市灯光区域发出对应颜色的光。

## 核心原理

三个组件组合即可实现：

```
ImageTexture("照片.jpg")  →  DiffuseLight::new_with_texture(...)  →  贴到 Quad/Sphere 上
```

| 组件 | 作用 | 定义位置 |
|------|------|----------|
| `ImageTexture` | 从磁盘加载图片，通过 UV 坐标采样像素颜色 | `src/tools/texture.rs:107-146` |
| `DiffuseLight<T>` | 泛型自发光材质，将纹理颜色直接作为发光色输出 | `src/tools/material.rs:122-158` |
| `Quad` / `Sphere` | 提供击中点的 UV 坐标 `(rec.u, rec.v)` | `src/tools/quad.rs:64-72` |

### 渲染管线数据流

```
光线击中几何体 (hit)
  → 获得 UV 坐标 (rec.u, rec.v)
  → 调用 mat.emitted(u, v, p)
    → DiffuseLight 内部调用 tex.value(u, v, p)
      → ImageTexture 根据 UV 采样照片像素
        → 返回该像素的 RGB 作为发光色
  → ray_color() 中：DiffuseLight::scatter() 返回 false → 直接返回发光色
```

### 关键代码

**DiffuseLight 泛型定义**（`src/tools/material.rs:122-158`）：

```rust
pub struct DiffuseLight<T: Texture = SolidColor> {
    tex: Arc<T>,
}

impl<T: Texture> DiffuseLight<T> {
    pub fn new_with_texture(texture: Arc<T>) -> Self {
        Self { tex }
    }
}

impl<T: Texture> Material for DiffuseLight<T> {
    fn scatter(&self, ..) -> bool {
        false  // 不自散射，光线终止
    }

    fn emitted(&self, u: f64, v: f64, p: Point3) -> Color {
        self.tex.value(u, v, p)  // 纹理颜色 = 发光颜色
    }
}
```

**ImageTexture 的 UV 采样**（`src/tools/texture.rs:124-146`）：

```rust
impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: Point3) -> Color {
        // 钳制 UV 到 [0,1]
        let u = Interval::new(0.0, 1.0).clamp(u);
        let v = 1.0 - Interval::new(0.0, 1.0).clamp(v); // 翻转 V 匹配图像坐标系

        let i = (u * self.image.width() as f64) as u32;
        let j = (v * self.image.height() as f64) as u32;
        let pixel = self.image.pixel_data(i, j);

        let color_scale = 1.0 / 255.0;
        Color::new(
            color_scale * pixel[0] as f64,
            color_scale * pixel[1] as f64,
            color_scale * pixel[2] as f64,
        )
    }
}
```

**Quad 的 UV 坐标设置**（`src/tools/quad.rs:64-72`）：

```rust
fn is_interior(a: f64, b: f64, rec: &mut HitRecord) -> bool {
    if !(0.0..=1.0).contains(&a) || !(0.0..=1.0).contains(&b) {
        return false;
    }
    rec.u = a;  // UV 的 u 坐标
    rec.v = b;  // UV 的 v 坐标
    true
}
```

## 使用示例

基于 `src/work/earth_night.rs` 的最小发光照片平面场景：

```rust
// 1. 加载照片纹理
let night_tex = Arc::new(ImageTexture::new("images/earth_night.jpg"));

// 2. 创建照片光源材质
let light_mat = Arc::new(DiffuseLight::new_with_texture(night_tex));

// 3. 贴到四边形平面上（左下角起点，u 向右 3 单位，v 向上 3 单位）
world.add(Box::new(Quad::new(
    Point3::new(-1.5, -1.5, -1.0),  // 左下角
    Vec3::new(3.0, 0.0, 0.0),       // u 边（向右）
    Vec3::new(0.0, 3.0, 0.0),       // v 边（向上）
    light_mat,
)));

// 4. 相机正对照片平面
cam.lookfrom = Point3::new(0.0, 0.0, -3.0);
cam.lookat = Point3::new(0.0, 0.0, -1.0);
```
