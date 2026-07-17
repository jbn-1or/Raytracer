# Static Dispatch 优化

## 原理

### 动态分发 (Dynamic Dispatch) 的开销

在 Rust 中，`Box<dyn Trait>`、`Arc<dyn Trait>` 和 `&dyn Trait` 都是 **trait object**。编译器为每种实现了该 trait 的具体类型生成一张 **虚函数表（vtable）**，trait object 内部存储两个指针：

1. **数据指针** — 指向堆上的实际数据
2. **虚表指针** — 指向该类型的 vtable

每次通过 trait object 调用方法时，需要：

```
读取虚表指针 → 跳转到虚表中的函数入口 → 执行调用
```

这引入了 **两次额外的内存间接访问**（读取虚表指针 + 读取虚表中的函数地址），CPU 分支预测器无法提前预测间接跳转目标，可能触发流水线冲刷。

### 静态分发 (Static Dispatch) 的优势

使用泛型时，Rust 编译器执行 **单态化（monomorphization）**：为每种实际使用的具体类型参数生成一份特化的代码副本。例如：

```rust
// 泛型版本
struct Lambertian<T: Texture> { tex: Arc<T> }
impl<T: Texture> Material for Lambertian<T> { ... }

// 使用时
let mat = Arc::new(Lambertian::<SolidColor>::new(color));
// 编译器生成 Lambertian<SolidColor>::scatter() 的特化版本
// 方法体中 self.tex.value() → 直接调用 SolidColor::value()
```

这带来的优势：
- **零间接调用** — 方法调用目标在编译时已知，生成直接的函数调用指令
- **内联优化** — 编译器可以将短函数内联到调用点，减少函数调用开销
- **CPU 分支预测友好** — 直接跳转目标可被预取

### 在本项目中的应用

热路径是光线的递归追踪（`ray_color` → `world.hit()` → `rec.mat.scatter()` → `self.tex.value()`）：

| 路径层级 | 类型 | 是否可静态分发 |
|----------|------|:---:|
| `world.hit()` | `&dyn Hittable` (HittableList/BVHNode) | ❌ 必须 dyn（异质集合） |
| `rec.mat.scatter()` | `Arc<dyn Material>` (HitRecord 中) | ❌ 必须 dyn（运行时多态） |
| `self.tex.value()` | `Arc<T: Texture>` (材质内部) | ✅ 静态分发 |

**关键洞察**：每次光线击中都会执行 `mat.scatter()`，散射计算内部会调用 `self.tex.value()` 查询纹理。虽然 `mat.scatter()` 本身必须通过虚表分发，但一旦进入具体材质的方法体，**纹理查询可以是单态化的**。

## 实现

### 1. 泛型材质

在 `src/tools/material.rs` 中，将依赖纹理的材质改为泛型：

```rust
/// 朗伯（漫反射）材质 — 泛型版本
pub struct Lambertian<T: Texture = SolidColor> {
    tex: Arc<T>,  // 具体纹理类型，非 trait object
}

// 默认使用 SolidColor 纹理
impl Lambertian<SolidColor> {
    pub fn new(albedo: Color) -> Self {
        Self { tex: Arc::new(SolidColor::from_color(albedo)) }
    }
}

// 接受任意纹理类型
impl<T: Texture> Lambertian<T> {
    pub fn new_with_texture(texture: Arc<T>) -> Self {
        Self { tex: texture }
    }
}

// Material trait 实现
impl<T: Texture> Material for Lambertian<T> {
    fn scatter(&self, ..., attenuation: &mut Color, ...) -> bool {
        // self.tex.value(...) → 编译器单态化为 SolidColor::value() 的直接调用
        *attenuation = self.tex.value(rec.u, rec.v, rec.p);
        true
    }
}
```

同样改造了 `DiffuseLight<T>` 和 `Isotropic<T>`。`Metal` 和 `Dielecric` 不含纹理字段，保持不变。

**效果**：`Lambertian::<SolidColor>::scatter()` 中调用 `self.tex.value()` → 编译为对 `SolidColor::value()` 的直接函数调用，无虚表间接访问。

### 2. 泛型变换

在 `src/tools/hittable.rs` 中：

```rust
/// 平移变换 — 泛型版本
pub struct Translate<H: Hittable> {
    object: Arc<H>,  // 具体物体类型
    offset: Vec3,
    bbox: Aabb,
}

impl<H: Hittable> Hittable for Translate<H> {
    fn hit(&self, r: &Ray, ...) -> bool {
        // self.object.hit(...) → 编译器单态化为具体类型的直接调用
        let offset_r = Ray::new_with_time(r.origin() - self.offset, ...);
        if !self.object.hit(&offset_r, ..., rec) { return false; }
        rec.p += self.offset;
        true
    }
}
```

同样改造了 `RotateY<H>`。当 `H` 是 `HittableList` 时，`Translate::<HittableList>::hit()` → `HittableList::hit()` 是单态化直接调用。

### 3. 物体构造器保持 dyn 兼容

`Sphere` 和 `Quad` 内部的 `mat` 字段保持 `Option<Arc<dyn Material>>`，因为最终必须存入 `HitRecord`。但构造函数使用泛型参数接收具体材质类型：

```rust
// Sphere 的泛型构造器（供静态分发使用）
pub fn new_with_material_static<M: Material + 'static>(
    center: Point3, radius: f64, mat: Arc<M>
) -> Self {
    Self {
        mat: Some(mat as Arc<dyn Material>),  // 在边界处一次性转换
        ...
    }
}
```

### 4. 场景代码优化

在 `image23.rs` 和 `image2_22.rs` 中，移除了所有 `Arc<dyn Material>` 中间变量绑定，改为直接内联具体类型：

```rust
// 优化前（需要 Arc<dyn Material> 中间变量）
let material: Arc<dyn Material> = Arc::new(Lambertian::new(color));
world.add(Box::new(Sphere::new_with_material(center, radius, material)));

// 优化后（具体类型直接转换）
world.add(Box::new(Sphere::new_with_material(
    center, radius,
    Arc::new(Lambertian::new(color)),  // Arc<Lambertian<SolidColor>> → Arc<dyn Material> 隐式转换
)));
```

`image2_22.rs` 中的 `ConstantMedium` 改用 `new_static` 泛型构造器：

```rust
world.add(Box::new(ConstantMedium::new_static(
    Arc::new(box1),
    0.01,
    Arc::new(Isotropic::new(Color::new(0.0, 0.0, 0.0))),  // 具体类型，非 dyn
)));
```

## 保留 dyn 的位置

按任务要求，以下位置保持使用 `dyn`（这些是异质集合或运行时多态的必经之路）：

| 位置 | 类型 | 原因 |
|------|------|------|
| `HitRecord.mat` | `Option<Arc<dyn Material>>` | 运行时从任意物体获取材质，无法在编译时确定类型 |
| `HittableList.objects` | `Vec<Box<dyn Hittable>>` | 异质集合，包含球体、四边形、变换、介质等不同类型 |
| `BVHNode.left` / `BVHNode.right` | `Arc<dyn Hittable>` | 左右子树类型可能不同 |
| `ray_color(world: &dyn Hittable)` | `&dyn Hittable` | 渲染入口必须是 trait object |

## 实际收益总结

| 优化点 | 消除的虚调用 |
|--------|:---:|
| `Lambertian::scatter()` 内 `tex.value()` | ✅ 虚表 → 直接调用 |
| `DiffuseLight::emitted()` 内 `tex.value()` | ✅ 虚表 → 直接调用 |
| `Isotropic::scatter()` 内 `tex.value()` | ✅ 虚表 → 直接调用 |
| `Translate::hit()` 内 `object.hit()` | ✅ 虚表 → 直接调用 |
| `RotateY::hit()` 内 `object.hit()` | ✅ 虚表 → 直接调用 |
| `Sphere::hit()` 内 `rec.mat` 赋值 | ⚠️ 仍需 `Arc<dyn Material>` |
| `BVHNode::hit()` 内递归 | ⚠️ 仍需 `Arc<dyn Hittable>` |

在每像素数百次光线追踪的渲染过程中，纹理查询和变换查询的静态分发消除了大量不必要的虚表间接访问，提升了渲染效率。