#![allow(dead_code)]

use crate::tools::color::Color;
use crate::tools::hittable::HitRecord;
use crate::tools::ray::Ray;
use crate::tools::vector3::{dot, random_unit_vector, reflect, refract, unit_vector};

/// 材质抽象接口，定义散射行为与衰减
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

        *scattered = Ray::new(rec.p, scatter_direction);
        *attenuation = self.albedo;
        true
    }
}

/// 镜面反射材质（金属）
pub struct Metal {
    /// 材质的反照率（颜色）
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    /// 创建金属材质
    /// # 参数`albedo`-材质的颜色
    pub fn new(albedo: Color) -> Self {
        Self { albedo, fuzz: 0.0 }
    }

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
        *scattered = Ray::new(rec.p, reflected);
        *attenuation = self.albedo;
        dot(scattered.direction(), rec.normal) > 0.0
    }
}

pub struct Dielecric {
    refraction_index: f64,
}

impl Dielecric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }
}

impl Material for Dielecric {
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
        let refracted = refract(unit_direction, rec.normal, ri);

        *scattered = Ray::new(rec.p, refracted);
        true
    }
}
