use std::num::Float;
use std::f32::consts::PI_2;

use rand;
use rand::distributions::IndependentSample;
use rand::Rng;

use point::Point3;
use vec::Vec3;
use ray::Ray3;
use distribution::{Distribution, Const};

pub struct Material<Refl> {
    reflection: Refl,
    emitted_radiance: f32
}

impl<Refl> Material<Refl> {
    pub fn new(reflection: Refl, emitted_radiance: f32) -> Material<Refl> {
        Material {
            reflection: reflection,
            emitted_radiance: emitted_radiance
        }
    }

    pub fn emitted(&self) -> f32 {
        self.emitted_radiance
    }

    pub fn reflection(&self) -> &Refl {
        &self.reflection
    }
}

pub trait Reflection<ObjectId> {
    type OutDist: Distribution<Output=(f32, Ray3, ObjectId)>;
    fn reflect(&self, incoming: Vec3, normal: Vec3, point: Point3, object: ObjectId) -> Self::OutDist;
}

pub struct Diffuse;

impl<ObjectId: Clone> Reflection<ObjectId> for Diffuse {
    type OutDist = DiffuseDist<ObjectId>;
    fn reflect(&self, _incoming: Vec3, normal: Vec3, point: Point3, object: ObjectId) -> DiffuseDist<ObjectId> {
        DiffuseDist {
            point: point,
            normal: normal,
            object: object
        }
    }
}

pub struct DiffuseDist<ObjectId> {
    point: Point3,
    normal: Vec3,
    object: ObjectId
}

impl<ObjectId: Clone> Distribution for DiffuseDist<ObjectId> {
    type Output = (f32, Ray3, ObjectId);

    fn sample<R: Rng>(&self, rng: &mut R) -> (f32, Ray3, ObjectId) {
        fn rand_sphere<R: rand::Rng>(rng: &mut R) -> Vec3 {
           let z = rand::distributions::Range::new(-1., 1.).ind_sample(rng);
           let r = (1. - z*z).sqrt();
           let angle = rand::distributions::Range::new(0., PI_2).ind_sample(rng);
           Vec3::new(r*angle.cos(), r*angle.sin(), z)
       }

        let cand_dir = rand_sphere(rng);
        let dir = if cand_dir.dot(self.normal) < 0. {
            -cand_dir
        } else {
            cand_dir
        };

        let scale = dir.dot(self.normal) / self.normal.mag2().sqrt();

        (
            scale,
            Ray3 {
                start: self.point,
                dir: dir
            },
            self.object.clone()
        )
    }
}

pub struct Specular;

impl<ObjectId: Clone> Reflection<ObjectId> for Specular {
    type OutDist = Const<(f32, Ray3, ObjectId)>;
    fn reflect(&self, incoming: Vec3, normal: Vec3, point: Point3, object: ObjectId) -> Const<(f32, Ray3, ObjectId)> {
        let projected = normal * incoming.dot(normal) / normal.mag2();
        Const::new((
            1.,
            Ray3 {
                start: point,
                dir: -incoming - projected * 2.
            },
            object
        ))
    }
}
