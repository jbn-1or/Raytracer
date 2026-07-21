#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::hittable::Hittable;
use crate::tools::material::Material;
use crate::tools::triangle::Triangle;
use crate::tools::vector3::{Point3, Vec3};

/// 加载 .obj 文件，为每个三角形面创建一个 Triangle 对象
///
/// # 参数
/// - `path`: .obj 文件路径
/// - `mat`: 应用于所有三角形的默认材质
///
/// # 返回值
/// 返回 `Vec<Box<dyn Hittable>>`，可直接传入 `HittableList`
pub fn load_obj(path: &str, mat: Arc<dyn Material>) -> Vec<Box<dyn Hittable>> {
    let (models, _materials) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,  // 将多边形面三角化
            single_index: true, // 使用单索引缓存
            ..Default::default()
        },
    )
    .unwrap_or_else(|_| panic!("Failed to load OBJ file: {}", path));

    let mut triangles: Vec<Box<dyn Hittable>> = Vec::new();

    for model in models.iter() {
        let mesh = &model.mesh;

        // 获取顶点数组
        let positions: Vec<Point3> = mesh
            .positions
            .chunks(3)
            .map(|p| Point3::new(p[0] as f64, p[1] as f64, p[2] as f64))
            .collect();

        // 获取法线数组（如果有）
        let normals: Option<Vec<Vec3>> = if mesh.normals.is_empty() {
            None
        } else {
            Some(
                mesh.normals
                    .chunks(3)
                    .map(|n| Vec3::new(n[0] as f64, n[1] as f64, n[2] as f64))
                    .collect(),
            )
        };

        // 遍历所有面（三角形，因为已开启三角化）
        for face in mesh.indices.chunks(3) {
            if face.len() < 3 {
                continue;
            }

            let i0 = face[0] as usize;
            let i1 = face[1] as usize;
            let i2 = face[2] as usize;

            let v0 = positions[i0];
            let v1 = positions[i1];
            let v2 = positions[i2];

            // 如果有法线数据，使用逐顶点法线创建三角形（平滑着色）
            let tri = if let Some(ref ns) = normals {
                let n0 = ns[i0];
                let n1 = ns[i1];
                let n2 = ns[i2];
                Triangle::new_with_normals(v0, v1, v2, n0, n1, n2)
            } else {
                Triangle::new(v0, v1, v2)
            };

            triangles.push(Box::new(tri.with_material(mat.clone())));
        }
    }

    triangles
}

/// 加载 .obj 文件并应用变换（缩放 + 平移 + 绕 Y 旋转），一步到位
///
/// # 参数
/// - `path`: .obj 文件路径
/// - `mat`: 默认材质
/// - `scale`: 缩放因子（均匀缩放）
/// - `translate`: 平移向量
/// - `rotate_y_deg`: 绕 Y 轴旋转的角度（角度制）
pub fn load_obj_transformed(
    path: &str,
    mat: Arc<dyn Material>,
    scale: f64,
    translate: Vec3,
    rotate_y_deg: f64,
) -> Vec<Box<dyn Hittable>> {
    let (models, _materials) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
    .unwrap_or_else(|_| panic!("Failed to load OBJ file: {}", path));

    let radians = rotate_y_deg.to_radians();
    let sin_theta = radians.sin();
    let cos_theta = radians.cos();

    let mut triangles: Vec<Box<dyn Hittable>> = Vec::new();

    for model in models.iter() {
        let mesh = &model.mesh;

        // 获取原始顶点数组
        let raw_positions: Vec<Point3> = mesh
            .positions
            .chunks(3)
            .map(|p| Point3::new(p[0] as f64, p[1] as f64, p[2] as f64))
            .collect();

        // 对每个顶点应用变换：缩放 → 绕 Y 旋转 → 平移
        let positions: Vec<Point3> = raw_positions
            .iter()
            .map(|p| {
                // 缩放
                let sx = p.x() * scale;
                let sy = p.y() * scale;
                let sz = p.z() * scale;
                // 绕 Y 旋转
                let rx = cos_theta * sx + sin_theta * sz;
                let ry = sy;
                let rz = -sin_theta * sx + cos_theta * sz;
                // 平移
                Point3::new(rx + translate.x(), ry + translate.y(), rz + translate.z())
            })
            .collect();

        // 获取法线数组，同样对法线应用旋转（法线不受缩放和平移影响，只需旋转）
        let normals: Option<Vec<Vec3>> = if mesh.normals.is_empty() {
            None
        } else {
            Some(
                mesh.normals
                    .chunks(3)
                    .map(|n| {
                        let nx = n[0] as f64;
                        let ny = n[1] as f64;
                        let nz = n[2] as f64;
                        // 法线只需要旋转变换
                        Vec3::new(
                            cos_theta * nx + sin_theta * nz,
                            ny,
                            -sin_theta * nx + cos_theta * nz,
                        )
                    })
                    .collect(),
            )
        };

        // 遍历所有面
        for face in mesh.indices.chunks(3) {
            if face.len() < 3 {
                continue;
            }

            let i0 = face[0] as usize;
            let i1 = face[1] as usize;
            let i2 = face[2] as usize;

            let v0 = positions[i0];
            let v1 = positions[i1];
            let v2 = positions[i2];

            let tri = if let Some(ref ns) = normals {
                let n0 = ns[i0];
                let n1 = ns[i1];
                let n2 = ns[i2];
                Triangle::new_with_normals(v0, v1, v2, n0, n1, n2)
            } else {
                Triangle::new(v0, v1, v2)
            };

            triangles.push(Box::new(tri.with_material(mat.clone())));
        }
    }

    triangles
}
