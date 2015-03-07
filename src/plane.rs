use point::Point3;
use vec::Vec3;
use ray::Ray3;
use material::{Material, Reflection};
use scene::{Scene, Intersection};

pub struct Plane<Refl> {
    origin: Point3,
    normal: Vec3,
    material: Material<Refl>
}

impl<Refl> Plane<Refl> {
    pub fn new(origin: Point3, normal: Vec3, material: Material<Refl>) -> Plane<Refl> {
        Plane {
            origin: origin,
            normal: normal,
            material: material
        }
    }
}

impl<Refl: Reflection<()>> Scene for Plane<Refl> {
    type ObjectId = ();
    type OutDist = Refl::OutDist;
    fn intersect(&self, ray: Ray3, previous: Option<()>) -> Option<Intersection<Refl::OutDist>> {
        if let Some(()) = previous {
            return None;
        }

        let divisor = ray.dir.dot(self.normal);
        if divisor == 0. {
            None
        } else {
            let offset = ray.start - self.origin;
            let time = -offset.dot(self.normal) / divisor;
            if time > 0. {
                let point = ray.start + ray.dir * time;
                Some(Intersection {
                    time: time,
                    emitted: self.material.emitted(),
                    reflection: self.material.reflection().reflect(
                        ray.dir,
                        self.normal,
                        point,
                        ()
                    )
                })
            } else {
                None
            }
        }
    }
}
