#![allow(dead_code)]

use crate::tools::color::Color;
use crate::tools::hittable::HitRecord;
use crate::tools::ray::Ray;
use crate::tools::rtweekend::random_double;
use crate::tools::vector3::{Vec3, dot, random_unit_vector, reflect, refract, unit_vector};

/// 材质 trait，定义光线与表面交互时的散射行为
pub trait Material {
    /// 产生散射光线（或说明它吸收了入射光线），若散射，说明光线应该衰减多少
    /// # 返回值：true 表示光线被散射，false 表示光线被吸收
    fn scatter(
        &self,
        _r_in: &Ray,
        _rec: &HitRecord,
        _attenuation: &mut Color,
        _scattered: &mut Ray,
    ) -> bool {
        false
    }
}

/// 朗伯（漫反射）材质
pub struct Lambertian {
    /// 材质的反照率（颜色）
    albedo: Color,
}

impl Lambertian {
    /// 创建朗伯材质
    /// # 参数`albedo`-材质的反照率颜色
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    /// 计算朗伯材质的散射光线和衰减
    /// 散射方向 = 法线 + 随机单位向量；若方向退化为零向量则回退为法线方向
    /// # 返回值：始终返回 true（光线总是被散射，不会吸收）
    fn scatter(
        &self,
        _r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let mut scatter_direction = rec.normal + random_unit_vector();

        // 捕获退化散射方向（零向量）
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        *scattered = Ray::new_with_time(rec.p, scatter_direction, _r_in.time());
        *attenuation = self.albedo;
        true
    }
}

/// 镜面反射材质（金属）
pub struct Metal {
    /// 材质的反照率（颜色）
    albedo: Color,
    /// 表面粗糙度（0=完美镜面，1=极度粗糙），使反射方向加入随机扰动
    fuzz: f64,
}

impl Metal {
    /// 创建金属材质
    /// # 参数`albedo`-材质的颜色
    pub fn new(albedo: Color) -> Self {
        Self { albedo, fuzz: 0.0 }
    }

    /// 创建带粗糙度的金属材质，`fuzz` 超出 [0,1] 时自动钳位到 1
    /// # 参数`albedo`-材质的颜色 `fu`-粗糙度（0~1）
    pub fn new_with_fuzz(albedo: Color, fu: f64) -> Self {
        let mut fuz = fu;
        if fuz > 1.0 {
            fuz = 1.0
        }
        Self { albedo, fuzz: fuz }
    }
}

impl Material for Metal {
    /// 计算金属材质的散射光线和衰减
    /// 光线沿法线反射
    /// # 返回值：光线是否被反射
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let mut reflected = reflect(r_in.direction(), rec.normal);
        reflected = unit_vector(reflected) + (self.fuzz * random_unit_vector());
        *scattered = Ray::new_with_time(rec.p, reflected, r_in.time());
        *attenuation = self.albedo;
        dot(scattered.direction(), rec.normal) > 0.0
    }
}

/// 电介质材质（如玻璃、水），同时支持折射和菲涅尔反射
pub struct Dielecric {
    /// 材质的折射率（相对于真空），常见值：玻璃≈1.5，水≈1.33
    refraction_index: f64,
}

impl Dielecric {
    /// 创建电介质材质
    /// # 参数`refraction_index`-折射率（通常 >1）
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }
}

/// 使用 Schlick 近似计算菲涅尔反射率，决定反射与折射的能量分配
/// # 参数`cosine`-入射角余弦值 `refraction_index`-相对折射率
fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
    let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

impl Material for Dielecric {
    /// 电介质材质的散射：根据入射角度和折射率决定反射或折射，模拟全内反射与菲涅尔效应
    /// # 返回值：始终返回 true（光线总是被散射，不会吸收）
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        *attenuation = Color::new(1.0, 1.0, 1.0);
        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = unit_vector(r_in.direction());
        let cos_theta = f64::min(dot(-unit_direction, rec.normal), 1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction: Vec3 = if cannot_refract || reflectance(cos_theta, ri) > random_double() {
            reflect(unit_direction, rec.normal)
        } else {
            refract(unit_direction, rec.normal, ri)
        };

        *scattered = Ray::new_with_time(rec.p, direction, r_in.time());
        true
    }
}
