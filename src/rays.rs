//!A raytracer.
#![feature(core, old_path, old_io)]
extern crate image;
extern crate rand;

use std::old_io::File;
use std::num::Float;
use std::f32::consts::PI_2;

use rand::distributions::IndependentSample;
use rand::Rng;

use point::Point3;
use vec::Vec3;

mod vec;
mod point;

#[derive(Copy)]
struct Ray3 {
    start: Point3,
    dir: Vec3
}

struct Intersection<ObjectId> {
    time: f32,
    point: Point3,
    normal: Vec3,
    emitted: f32,
    reflection: Reflection,
    object: ObjectId
}

impl<ObjectId> Intersection<ObjectId> {
    fn map_obj<NewObjectId, F: FnOnce(ObjectId) -> NewObjectId>(self, func: F) -> Intersection<NewObjectId> {
        Intersection {
            time: self.time,
            point: self.point,
            normal: self.normal,
            emitted: self.emitted,
            reflection: self.reflection,
            object: func(self.object)
        }
    }
}

trait Scene {
    // TODO: Should this have a "not present" id?
    type ObjectId: Copy;

    fn intersect(&self, ray: Ray3, previous: Option<Self::ObjectId>) -> Option<Intersection<Self::ObjectId>>;
}

#[derive(Copy)]
struct Material {
    reflection: Reflection,
    emitted_radiance: f32
}

#[derive(Copy)]
enum Reflection {
    Diffuse,
    Specular
}

struct Sphere {
    center: Point3,
    radius: f32,
    material: Material
}

#[derive(Copy)]
enum SphereSource {
    Inside,
    Outside
}

impl Scene for Sphere {
    type ObjectId = SphereSource;
    fn intersect(&self, ray: Ray3, previous: Option<SphereSource>) -> Option<Intersection<SphereSource>> {
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
            point: p,
            normal: normal,
            emitted: self.material.emitted_radiance,
            reflection: self.material.reflection,
            object: match previous {
                None => SphereSource::Outside,
                Some(source) => source
            }
        })
    }
}

struct Plane {
    origin: Point3,
    normal: Vec3,
    material: Material
}

impl Scene for Plane {
    type ObjectId = ();
    fn intersect(&self, ray: Ray3, previous: Option<()>) -> Option<Intersection<()>> {
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
                Some(Intersection {
                    time: time,
                    point: ray.start + ray.dir * time,
                    normal: self.normal,
                    emitted: self.material.emitted_radiance,
                    reflection: self.material.reflection,
                    object: ()
                })
            } else {
                None
            }
        }
    }
}

#[derive(Copy)]
enum Choice<A, B> {
    OptA(A),
    OptB(B)
}

impl<A: Scene, B: Scene> Scene for (A, B) {
    type ObjectId = Choice<A::ObjectId, B::ObjectId>;
    fn intersect(&self, ray: Ray3, previous: Option<Choice<A::ObjectId, B::ObjectId>>) -> Option<Intersection<Choice<A::ObjectId, B::ObjectId>>> {
        let (aid, bid) = match previous {
            None => (None, None),
            Some(Choice::OptA(aid)) => (Some(aid), None),
            Some(Choice::OptB(bid)) => (None, Some(bid))
        };

        match (self.0.intersect(ray, aid), self.1.intersect(ray, bid)) {
            (Some(ai), Some(bi)) => if ai.time < bi.time {
                Some(ai.map_obj(Choice::OptA))
            } else {
                Some(bi.map_obj(Choice::OptB))
            },
            (Some(ai), None) => Some(ai.map_obj(Choice::OptA)),
            (None, Some(bi)) => Some(bi.map_obj(Choice::OptB)),
            (None, None) => None
        }
    }
}

impl<T: Scene> Scene for [T] {
    type ObjectId = (usize, T::ObjectId);
    fn intersect(&self, ray: Ray3, previous: Option<(usize, T::ObjectId)>) -> Option<Intersection<(usize, T::ObjectId)>> {
        let mut best: Option<Intersection<(usize, T::ObjectId)>> = None;

        for (id, obj) in self.iter().enumerate() {
            let prev = match previous {
                Some((prev_index, prev_obj)) => if prev_index == id {
                    Some(prev_obj)
                } else {
                    None
                },
                None => None
            };

            if let Some(intersection) = obj.intersect(ray, prev) {
                let new_best = match best {
                    Some(ref cur_best) => intersection.time < cur_best.time,
                    None => true
                };

                if new_best {
                    best = Some(intersection.map_obj(|obj| (id, obj)));
                }
            }
        }

        best
    }
}

impl<'a, T: ?Sized + Scene> Scene for &'a T {
    type ObjectId = T::ObjectId;
    fn intersect(&self, ray: Ray3, previous: Option<T::ObjectId>) -> Option<Intersection<T::ObjectId>> {
        (*self).intersect(ray, previous)
    }
}

fn clamp(val: f32) -> f32 {
    if val > 1. {
        1.
    } else if val < 0. {
        0.
    } else {
        val
    }
}

fn rand_sphere<R: rand::Rng>(rng: &mut R) -> Vec3 {
    let z = rand::distributions::Range::new(-1., 1.).ind_sample(rng);
    let r = (1. - z*z).sqrt();
    let angle = rand::distributions::Range::new(0., PI_2).ind_sample(rng);
    Vec3::new(r*angle.cos(), r*angle.sin(), z)
}

fn main() {
    let mut rng = rand::weak_rng();

    let rays_per_pixel = 200;
    let scene: (&[Sphere], &[Plane]) = (
        &[Sphere {
            center: Point3::new(1.2, 0.0, 5.0),
            radius: 0.3,
            material: Material { reflection: Reflection::Diffuse, emitted_radiance: 0. }
        },
        Sphere {
            center: Point3::new(1.5, 0.0, 3.5),
            radius: 0.35,
            material: Material { reflection: Reflection::Diffuse, emitted_radiance: 0. }
        },
        Sphere {
            center: Point3::new(1.0, 2.5, 0.5),
            radius: 1.0,
            material: Material { reflection: Reflection::Diffuse, emitted_radiance: 20. }
        },
        Sphere {
            center: Point3::new(-1.5, 0.0, 3.5),
            radius: 1.0,
            material: Material { reflection: Reflection::Specular, emitted_radiance: 0. }
        }],
        &[Plane {
            origin: Point3::new(0.0, 0.0, 8.0),
            normal: Vec3::new(2.0, 0.0, -1.0),
            material: Material { reflection: Reflection::Diffuse, emitted_radiance: 0. }
        },
        Plane {
            origin: Point3::new(0.0, 0.0, 8.0),
            normal: Vec3::new(-1.0, 0.0, -2.0),
            material: Material { reflection: Reflection::Diffuse, emitted_radiance: 0. }
        },
        Plane {
            origin: Point3::new(0.0, 2.0, 0.0),
            normal: Vec3::new(0.0, -1.0, 0.0),
            material: Material { reflection: Reflection::Diffuse, emitted_radiance: 0. }
        },
        Plane {
            origin: Point3::new(0.0, -3.0, 0.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
            material: Material { reflection: Reflection::Diffuse, emitted_radiance: 0. }
        },
        Plane {
            origin: Point3::new(0.0, 0.0, -10.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            material: Material { reflection: Reflection::Diffuse, emitted_radiance: 0. }
        }]
    );

    let imgx = 800;
    let imgy = 800;

    let scalex = 4.0 / imgx as f32;
    let scaley = 4.0 / imgy as f32;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordiantes and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let cy = -(y as f32 * scaley - 2.0);
        let cx = x as f32 * scalex - 2.0;

        let mut total_light = 0.;
        for _ in 0..rays_per_pixel {
            let mut ray = Ray3 {
                start: Point3::new(0., 0., -8.0),
                dir: Vec3::new(cx, cy, 8.0)
            };
            let mut prev_object = None;
            let mut scale = 1.;
            loop {
                if let Some(intersection) = scene.intersect(ray, prev_object) {
                    prev_object = Some(intersection.object);

                    total_light += scale * intersection.emitted;

                    ray = match intersection.reflection {
                        Reflection::Specular => {
                            let projected = intersection.normal * ray.dir.dot(intersection.normal) / intersection.normal.mag2();
                            let new_dir = -ray.dir - projected * 2.;
                            Ray3 {
                                start: intersection.point,
                                dir: new_dir
                            }
                        },
                        Reflection::Diffuse => {
                            let cand_dir = rand_sphere(&mut rng);
                            let dir = if cand_dir.dot(intersection.normal) < 0. {
                                -cand_dir
                            } else {
                                cand_dir
                            };

                            let p = dir.dot(intersection.normal) / intersection.normal.mag2().sqrt();
                            if scale > 0.2 {
                                scale *= p;
                            } else if rng.next_f32() > p {
                                break;
                            }

                            Ray3 {
                                start: intersection.point,
                                dir: dir
                            }
                        }
                    }
                } else {
                    break;
                }
            }
        }

        // Create an 8bit pixel of type Luma and value i
        // and assign in to the pixel at position (x, y)
        *pixel = image::Luma([(clamp(total_light / rays_per_pixel as f32)*255.) as u8]);
    }

    // Save the image as “out.png”
    let ref mut fout = File::create(&Path::new("out.png")).unwrap();

    // We must indicate the image’s color type and what format to save as
    let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);
}
