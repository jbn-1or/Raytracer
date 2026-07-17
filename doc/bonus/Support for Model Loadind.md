# 3D 模型加载支持（.obj 文件）

## 原理

### 为什么需要三角形几何体

3D 模型本质上是一组三角形面片的集合。`.obj`（Wavefront OBJ）是业界最通用的 3D 模型文本格式，它存储：

- **顶点位置**：每个顶点的 3D 坐标 `(x, y, z)`
- **顶点法线**：每个顶点的法向量（可选，用于平滑着色）
- **纹理坐标**：UV 映射坐标（可选）
- **面（Face）**：由 3 个顶点索引组成的三角形（或更多顶点的多边形）

现有的 `Sphere` 和 `Quad` 几何体无法描述任意的三角形网格，因此需要实现 `Triangle` 几何体作为最基本的图元，与现有的光线追踪管线无缝集成。

### Möller–Trumbore 算法

使用 **Möller–Trumbore 光线-三角形求交算法**（Tomas Möller & Ben Trumbore, 1997），该算法用重心坐标直接求解光线与三角形平面的交点：

1. 计算光线方向与三角形两条边的叉积，得到行列式 `det`
2. 若 `|det|` 接近零，光线与三角形平面平行，无交点
3. 用 Cramer 法则求解重心坐标 `(u, v)`
4. 若 `u ≥ 0`、`v ≥ 0` 且 `u + v ≤ 1`，交点在三角形内部
5. 由 `(u, v)` 可直接插值法线（平滑着色）和 UV 坐标

算法的优势是无需预先计算平面方程，计算量小，且重心坐标天然适合纹理/法线插值。

### 与现有管线的集成

所有几何体只需实现 `Hittable` trait 的 `hit()` 和 `bounding_box()` 方法。`Triangle` 实现后，可以直接：

- 加入 `HittableList` 构建场景
- 被 `BvhNode::from_list()` 自动构建为 BVH 加速结构
- 与现有的 `Translate`、`RotateY` 和新增的 `Scale` 变换组合使用

## 实现

### 1. 添加依赖

```toml
# Cargo.toml
[dependencies]
tobj = "4"
```

`tobj` 是 Rust 社区最成熟的 `.obj` 加载库，支持顶点、法线、纹理坐标解析，以及自动三角化（将多边形面转为三角形）。

### 2. Triangle 几何体

新文件 `src/tools/triangle.rs`，实现三个核心能力：

**结构体定义**：
```rust
pub struct Triangle {
    vertices: [Point3; 3],           // 3 个顶点
    normals: Option<[Vec3; 3]>,       // 可选的逐顶点法线
    pub mat: Option<Arc<dyn Material>>,
    bbox: Aabb,
}
```

**Möller–Trumbore 求交**（`hit()` 方法）：
- 计算 `edge1 = v1 - v0`、`edge2 = v2 - v0`
- 叉积 `pvec = cross(direction, edge2)`，行列式 `det = dot(edge1, pvec)`
- `det` 过小 → 平行，返回 `false`
- 求解 `u` 和 `v`，边界检查
- 求解交点参数 `t`，范围检查
- 记录交点、法线和材质

**平滑着色**：若 `normals` 存在，用重心坐标插值：`n = n0 * w + n1 * u + n2 * v`（其中 `w = 1 - u - v`）；否则使用面法线 `cross(edge1, edge2).normalize()`。

**包围盒**：取 3 个顶点坐标的 min/max。

两个构造函数：
- `Triangle::new(v0, v1, v2)` — 面法线（硬边着色）
- `Triangle::new_with_normals(v0, v1, v2, n0, n1, n2)` — 逐顶点法线（平滑着色）
- `with_material(mat)` — 构建器模式设置材质

### 3. Scale 变换包装器

新文件 `src/tools/scale.rs`，与已有的 `Translate` 和 `RotateY` 风格一致：

```rust
pub struct Scale<H: Hittable> {
    object: Arc<H>,
    scale: f64,
    bbox: Aabb,
}
```

**光线的逆变换**：
1. 将光线原点除以 `scale`（变换到物体空间）
2. 在物体空间中求交
3. 将交点位置乘以 `scale`（变换回世界空间）
4. 法线方向不变，只需重新归一化

**包围盒**：将子物体包围盒的 min/max 各分量乘以 `scale`。

注意：非均匀缩放需要法线变换比 `M^{-T}`，但当前实现为各向同性缩放，法线方向不变。

### 4. OBJ 加载器

新文件 `src/tools/obj_loader.rs`，提供两个函数：

**`load_obj(path, mat)`** — 基础加载：
1. 调用 `tobj::load_obj()`，启用 `triangulate: true`（将多边形自动转为三角形）
2. 遍历所有子模型（`models`），提取 `positions` 数组
3. 如有法线数据（`normals` 非空），使用 `new_with_normals()` 创建三角形
4. 遍历面索引（每 3 个一组），创建 `Triangle` 并设置材质
5. 返回 `Vec<Box<dyn Hittable>>`

**`load_obj_transformed(path, mat, scale, translate, rotate_y_deg)`** — 带变换加载：
- 对每个顶点依次应用：**缩放 → 绕 Y 轴旋转 → 平移**
- 法线仅应用旋转变换（法线不受平移和均匀缩放影响）
- 变换直接作用于顶点坐标，避免运行时变换开销

### 5. 模块注册

在 `src/tools/mod.rs` 中添加：

```rust
pub mod triangle;
pub mod scale;
pub mod obj_loader;
```

## 使用方法

### 基础用法：加载 .obj 模型并加入场景

```rust
use std::sync::Arc;
use crate::tools::obj_loader;
use crate::tools::material::Lambertian;
use crate::tools::color::Color;
use crate::tools::hittable_list::HittableList;
use crate::tools::bvh::BvhNode;

// 加载模型
let triangles = obj_loader::load_obj(
    "assets/teapot.obj",
    Arc::new(Lambertian::new(Color::new(0.7, 0.3, 0.3))),
);

let mut world = HittableList::new();
for tri in triangles {
    world.add(tri);
}

// 必须用 BVH 加速（模型可能有成千上万个三角形）
let bvh = BvhNode::from_list(world);
```

### 变换加载：一步完成缩放 + 平移 + 旋转

```rust
use crate::tools::vector3::Vec3;

let bunny = obj_loader::load_obj_transformed(
    "assets/bunny.obj",
    Arc::new(Lambertian::new(Color::new(0.8, 0.6, 0.4))),
    10.0,                       // 缩放 10 倍
    Vec3::new(0.0, -2.0, 5.0),  // 平移
    45.0,                       // 绕 Y 轴旋转 45 度
);

for tri in bunny {
    world.add(tri);
}
```

### 与其他材质配合

任意实现了 `Material` trait 的材质都可以使用：

```rust
// 金属材质
Arc::new(Metal::new(Color::new(0.9, 0.9, 0.9)))

// 玻璃材质
Arc::new(Dielecric::new(1.5))

// 带纹理的漫反射
Arc::new(Lambertian::new_with_texture(checker_texture))
```

### 性能建议

- **必须配合 BVH**：模型通常有数千至数十万个三角形。不使用 BVH 时，每条光线需要遍历所有三角形（O(n)），渲染极慢。通过 `BvhNode::from_list()` 构建 BVH 后，求交复杂度降至 O(log n)。
- **选用 Release 模式**：`cargo run --release` 启用编译器优化。
- **并行渲染**：已集成的 Rayon 多线程自动加速三角形场景。

## 参考

- Möller, T., & Trumbore, B. (1997). *Fast, Minimum Storage Ray/Triangle Intersection*. Journal of Graphics Tools.
- [tobj - Rust 文档](https://docs.rs/tobj/latest/tobj/)
- [Wavefront .obj 文件格式规范](https://en.wikipedia.org/wiki/Wavefront_.obj_file)