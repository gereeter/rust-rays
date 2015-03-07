//!A raytracer.
#![feature(core, os, old_path, old_io)]
extern crate image;
extern crate rand;

use std::old_io::File;
use std::num::Float;
use std::f32::consts::PI_2;
use std::thread;
use std::os;

use rand::distributions::IndependentSample;
use rand::Rng;

use point::Point3;
use vec::Vec3;
use distribution::{Distribution, Const};

mod vec;
mod point;
mod distribution;

#[derive(Copy, Clone)]
struct Ray3 {
    start: Point3,
    dir: Vec3
}

struct Intersection<OutDist> {
    time: f32,
    emitted: f32,
    reflection: OutDist
}

impl<OutDist> Intersection<OutDist> {
    fn map_dist<NewOutDist, F: FnOnce(OutDist) -> NewOutDist>(self, func: F) -> Intersection<NewOutDist> {
        Intersection {
            time: self.time,
            emitted: self.emitted,
            reflection: func(self.reflection)
        }
    }
}

trait Scene {
    // TODO: Should this have a "not present" id?
    type ObjectId: Copy;
    type OutDist: Distribution<Output=(f32, Ray3, Self::ObjectId)>;

    fn intersect(&self, ray: Ray3, previous: Option<Self::ObjectId>) -> Option<Intersection<Self::OutDist>>;
}

#[derive(Copy)]
struct Material<Refl> {
    reflection: Refl,
    emitted_radiance: f32
}

struct Sphere<Refl> {
    center: Point3,
    radius: f32,
    material: Material<Refl>
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
            emitted: self.material.emitted_radiance,
            reflection: self.material.reflection.reflect(
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

struct Plane<Refl> {
    origin: Point3,
    normal: Vec3,
    material: Material<Refl>
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
                    emitted: self.material.emitted_radiance,
                    reflection: self.material.reflection.reflect(
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

#[derive(Copy)]
enum Choice<A, B> {
    OptA(A),
    OptB(B)
}

impl<AObj, BObj, A: Distribution<Output=(f32, Ray3, AObj)>, B: Distribution<Output=(f32, Ray3, BObj)>> Distribution for Choice<A, B> {
    type Output = (f32, Ray3, Choice<AObj, BObj>);

    fn sample<R: Rng>(&self, rng: &mut R) -> (f32, Ray3, Choice<AObj, BObj>) {
        match *self {
            Choice::OptA(ref a) => {
                let (scale, ray, obj) = a.sample(rng);
                (scale, ray, Choice::OptA(obj))
            },
            Choice::OptB(ref b) => {
                let (scale, ray, obj) = b.sample(rng);
                (scale, ray, Choice::OptB(obj))
            }
        }
    }
}

impl<A: Scene, B: Scene> Scene for (A, B) {
    type ObjectId = Choice<A::ObjectId, B::ObjectId>;
    type OutDist = Choice<A::OutDist, B::OutDist>;
    fn intersect(&self, ray: Ray3, previous: Option<Choice<A::ObjectId, B::ObjectId>>) -> Option<Intersection<Choice<A::OutDist, B::OutDist>>> {
        let (aid, bid) = match previous {
            None => (None, None),
            Some(Choice::OptA(aid)) => (Some(aid), None),
            Some(Choice::OptB(bid)) => (None, Some(bid))
        };

        match (self.0.intersect(ray, aid), self.1.intersect(ray, bid)) {
            (Some(ai), Some(bi)) => if ai.time < bi.time {
                Some(ai.map_dist(Choice::OptA))
            } else {
                Some(bi.map_dist(Choice::OptB))
            },
            (Some(ai), None) => Some(ai.map_dist(Choice::OptA)),
            (None, Some(bi)) => Some(bi.map_dist(Choice::OptB)),
            (None, None) => None
        }
    }
}

struct TagObject<T, Dist> {
    tag: T,
    dist: Dist
}

impl<T: Clone, O, Dist: Distribution<Output=(f32, Ray3, O)>> Distribution for TagObject<T, Dist> {
    type Output = (f32, Ray3, (T, O));

    fn sample<R: Rng>(&self, rng: &mut R) -> (f32, Ray3, (T, O)) {
        let (scale, ray, obj) = self.dist.sample(rng);
        (scale, ray, (self.tag.clone(), obj))
    }
}

impl<T: Scene> Scene for [T] {
    type ObjectId = (usize, T::ObjectId);
    type OutDist = TagObject<usize, T::OutDist>;
    fn intersect(&self, ray: Ray3, previous: Option<(usize, T::ObjectId)>) -> Option<Intersection<TagObject<usize, T::OutDist>>> {
        let mut best: Option<Intersection<TagObject<usize, T::OutDist>>> = None;

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
                    best = Some(intersection.map_dist(|dist| TagObject { tag: id, dist: dist }));
                }
            }
        }

        best
    }
}

impl<'a, T: ?Sized + Scene> Scene for &'a T {
    type ObjectId = T::ObjectId;
    type OutDist = T::OutDist;
    fn intersect(&self, ray: Ray3, previous: Option<T::ObjectId>) -> Option<Intersection<T::OutDist>> {
        (*self).intersect(ray, previous)
    }
}



trait Reflection<ObjectId> {
    type OutDist: Distribution<Output=(f32, Ray3, ObjectId)>;
    fn reflect(&self, incoming: Vec3, normal: Vec3, point: Point3, object: ObjectId) -> Self::OutDist;
}

struct Diffuse;

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

struct DiffuseDist<ObjectId> {
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

struct Specular;

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

fn clamp(val: f32) -> f32 {
    if val > 1. {
        1.
    } else if val < 0. {
        0.
    } else {
        val
    }
}

fn main() {
    let rays_per_pixel = 200;
    let scene: ((&[_],_),&[_]) = ((
        &[Sphere {
            center: Point3::new(1.2, 0.0, 5.0),
            radius: 0.3,
            material: Material { reflection: Diffuse, emitted_radiance: 0. }
        },
        Sphere {
            center: Point3::new(1.5, 0.0, 3.5),
            radius: 0.35,
            material: Material { reflection: Diffuse, emitted_radiance: 0. }
        },
        Sphere {
            center: Point3::new(1.0, 2.5, 0.5),
            radius: 1.0,
            material: Material { reflection: Diffuse, emitted_radiance: 30. }
        }],
        &Sphere {
            center: Point3::new(-1.5, 0.0, 3.5),
            radius: 1.0,
            material: Material { reflection: Specular, emitted_radiance: 0. }
        }),
        &[Plane {
            origin: Point3::new(0.0, 0.0, 8.0),
            normal: Vec3::new(2.0, 0.0, -1.0),
            material: Material { reflection: Diffuse, emitted_radiance: 0. }
        },
        Plane {
            origin: Point3::new(0.0, 0.0, 8.0),
            normal: Vec3::new(-1.0, 0.0, -2.0),
            material: Material { reflection: Diffuse, emitted_radiance: 0. }
        },
        Plane {
            origin: Point3::new(0.0, 2.0, 0.0),
            normal: Vec3::new(0.0, -1.0, 0.0),
            material: Material { reflection: Diffuse, emitted_radiance: 0. }
        },
        Plane {
            origin: Point3::new(0.0, -3.0, 0.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
            material: Material { reflection: Diffuse, emitted_radiance: 0. }
        },
        Plane {
            origin: Point3::new(0.0, 0.0, -10.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            material: Material { reflection: Diffuse, emitted_radiance: 0. }
        }]
    );

    let num_threads = os::num_cpus();

    let imgx = 800;
    let imgy = 800;

    let scalex = 4.0 / imgx as f32;
    let scaley = 4.0 / imgy as f32;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::<image::Luma<u8>>::new(imgx, imgy);

    // Iterate over the pixels in parallel
    {
        let mut img_data = imgbuf.as_mut_slice();
        let chunk_size = img_data.len() / num_threads + 1;
        img_data.chunks_mut(chunk_size).enumerate().map(|(chunk_i, chunk)| {
            thread::scoped(move || {
                let mut rng = rand::weak_rng();

                let base = (chunk_i * chunk_size) as u32;
                let mut x = base % imgx;
                let mut y = base / imgx;
                for pixel in chunk.iter_mut() {
                    x += 1;
                    if x == imgx {
                        x = 0;
                        y += 1;
                    }

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
                                total_light += scale * intersection.emitted;

                                let (reflected, new_ray, object) = intersection.reflection.sample(&mut rng);
                                ray = new_ray;
                                prev_object = Some(object);

                                if scale > 0.2 {
                                    scale *= reflected;
                                } else if rng.next_f32() > reflected {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }

                    // Create an 8bit pixel of type Luma and value i
                    // and assign in to the pixel at position (x, y)
                    *pixel = (clamp(total_light / rays_per_pixel as f32)*255.) as u8;
                }
            })
        }).collect::<Vec<_>>();
    }

    // Save the image as “out.png”
    let ref mut fout = File::create(&Path::new("out.png")).unwrap();

    // We must indicate the image’s color type and what format to save as
    let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);
}
