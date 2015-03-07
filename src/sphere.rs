use std::num::Float;

use point::Point3;
use ray::Ray3;
use material::{Material, Reflection};
use scene::{Scene, Intersection};

pub struct Sphere<Refl> {
    center: Point3,
    radius: f32,
    material: Material<Refl>
}

impl<Refl> Sphere<Refl> {
    pub fn new(center: Point3, radius: f32, material: Material<Refl>) -> Sphere<Refl> {
        Sphere {
            center: center,
            radius: radius,
            material: material
        }
    }
}

#[derive(Copy, Clone)]
enum SphereSource {
    Inside,
    Outside
}

impl<Refl: Reflection<SphereSource>> Scene for Sphere<Refl> {
    type ObjectId = SphereSource;
    type OutDist = Refl::OutDist;
    fn intersect(&self, ray: Ray3, previous: Option<SphereSource>) -> Option<Intersection<Refl::OutDist>> {
        if let Some(SphereSource::Outside) = previous {
            return None;
        }

        let offset = ray.start - self.center;

        let a = ray.dir.mag2();
        let b = 2. * offset.dot(ray.dir);
        let c = offset.mag2() - self.radius*self.radius;

        let descrim = b*b - 4.*a*c;
        if descrim < 0. {
            return None;
        }

        let time = {
            let t1 = (-b - descrim.sqrt()) / (2. * a);
            let t2 = (-b + descrim.sqrt()) / (2. * a);
            if previous.is_none() && t1 > 0. {
                t1
            } else if t2 > 0. {
                t2
            } else {
                return None;
            }
        };

        let p = ray.start + ray.dir * time;
        let normal = p - self.center;
        Some(Intersection {
            time: time,
            emitted: self.material.emitted(),
            reflection: self.material.reflection().reflect(
                ray.dir,
                normal,
                p,
                previous.unwrap_or(if c < 0. {
                    SphereSource::Inside
                } else {
                    SphereSource::Outside
                })
            )
        })
    }
}
