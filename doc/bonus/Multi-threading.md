# CPU 多线程并行加速（Rayon 实现）

## 原理

光线追踪的渲染循环由三重嵌套循环组成：

```
for 每一行 j:
    for 每一列 i:
        for 每个采样:
            生成并追踪光线
        写入像素颜色
```

每个像素的计算完全独立——不依赖相邻像素的结果。这是典型的 **Embarrassingly Parallel** 问题，天然适合 CPU 多线程并行加速。

### 并行化策略

采用 **按行并行** 策略：

1. **阶段 1（并行）**：将 `(0..height).into_par_iter()` 分配给所有 CPU 核心，每行内的像素串行计算，Rayon 通过 work-stealing 算法自动负载均衡
2. **阶段 2（串行）**：将计算好的像素颜色数组逐一写入图片缓冲区，确保写入顺序正确

### 线程安全性分析

| 组件 | 分析 |
|------|------|
| `cam.get_ray(i, j)` | 只接受 `&self`，内部 `rand::random()` 使用 thread-local RNG，天然线程安全 |
| `world`（场景） | `&dyn Hittable` 是不可变引用，所有物体只读 |
| `ray_color()` | 纯函数调用，无副作用 |
| `ProgressBar::inc()` | `indicatif` 内部使用原子计数器，并行安全 |
| 图片写入 | **串行执行**，不同线程不会同时写入同一像素 |

## 实现

### 1. 添加依赖

```toml
# Cargo.toml
[dependencies]
rayon = "1"
```

### 2. 通用并行渲染函数

在 `src/tools/render_utils.rs` 中添加：

```rust
/// 通用并行渲染函数
/// 阶段 1：按行并行计算所有像素颜色
/// 阶段 2：串行写入图片（保证进度条和写入顺序正确）
pub fn render_parallel<F, W>(
    img: &mut RgbImage,
    width: u32,
    height: u32,
    pixel_fn: F,          // 像素计算闭包：接收 (i, j) → Color
    write_pixel: W,       // 写入闭包：接收 (Color, &mut Rgb<u8>)
    progress: &ProgressBar,
)
where
    F: Fn(u32, u32) -> Color + Send + Sync,
    W: Fn(Color, &mut Rgb<u8>),
{
    use rayon::prelude::*;

    // 阶段 1：按行并行计算像素颜色
    let colors: Vec<Color> = (0..height)
        .into_par_iter()
        .flat_map(|j| (0..width).map(|i| pixel_fn(i, j)).collect::<Vec<_>>())
        .collect();

    // 阶段 2：串行写入图片
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel_mut(x, y);
            write_pixel(colors[(y * width + x) as usize], pixel);
            progress.inc(1);
        }
    }
}
```

并提供了两个便捷封装：
- `render_parallel_simple` — 使用 `write_color`（无 gamma 校正）
- `render_parallel_gamma` — 使用 `write_color_gamma`（带 gamma 校正）

### 3. 修改场景文件

将原来所有场景文件 `src/book1/*.rs` 和 `src/book2/*.rs` 中的渲染循环替换为调用并行渲染函数。

**改前**（串行）：
```rust
for j in 0..image_height {
    for i in 0..image_width {
        let mut pixel_color = Color::zero();
        for _sample in 0..cam.samples_per_pixel {
            let r = cam.get_ray(i, j);
            pixel_color += ray_color(&r, cam.max_depth, &world);
        }
        let pixel = img.get_pixel_mut(i, j);
        Color::write_color_gamma(pixel_color * pixel_samples_scale, pixel);
        progress.inc(1);
    }
}
```

**改后**（并行）：
```rust
let samples = cam.samples_per_pixel;
let max_depth = cam.max_depth;
render_parallel_gamma(&mut img, image_width, image_height, move |i, j| {
    let mut pixel_color = Color::zero();
    for _sample in 0..samples {
        let r = cam.get_ray(i, j);
        pixel_color += ray_color(&r, max_depth, &world);
    }
    pixel_color * pixel_samples_scale
}, &progress);
```

共修改了 **40 个场景文件**（book1 中 20 个 + book2 中 18 个 + 特殊场景 2 个）。

## 使用方法

### 启用并行加速

直接运行即可自动使用所有 CPU 核心：

```bash
cargo run --release
```

### 回退到单线程（用于性能对比）

设置环境变量 `RAYON_NUM_THREADS=1` 即可：

```bash
RAYON_NUM_THREADS=1 cargo run --release
```

### 控制使用的线程数

```bash
RAYON_NUM_THREADS=4 cargo run --release   # 使用 4 个线程
```

## 优化效果

### 测试环境
- **CPU**: AMD Ryzen 7 5800H (8C/16T)
- **Memory**: 16GB DDR4

### 测试场景 1：book2::image2_22
```
cam.aspect_ratio = 1.0;
cam.image_width = 900;
cam.samples_per_pixel = 500;
cam.max_depth = 50;
```

| 模式 | 耗时 |
|------|------|
| 优化前（单线程） | ~10min 30s |
| 优化后（16线程） | **待测试** |

### 测试场景 2：book1::image23
```
cam.aspect_ratio = 16.0 / 9.0;
cam.image_width = 1600;
cam.samples_per_pixel = 500;
cam.max_depth = 100;
```

| 模式 | 耗时 |
|------|------|
| 优化前（单线程） | ~8min |
| 优化后（16线程） | **待测试** |

> 实际加速比取决于 CPU 核心数和光线追踪场景的复杂度。
> 理想情况下，`16 核 CPU` 可达到约 **15x 加速**。

## 参考

- [Rayon 官方文档](https://docs.rs/rayon/latest/rayon/)
- [Rayon - Rayon: A data parallelism library for Rust](https://github.com/rayon-rs/rayon)