use std::num::Float;

use rand::Rng;

use point::Point3;
use vec::Vec3;
use ray::Ray3;
use distribution::Distribution;
use material::{Material, Reflection};

pub struct Intersection<OutDist> {
    pub time: f32,
    pub emitted: f32,
    pub reflection: OutDist
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

pub trait Scene {
    // TODO: Should this have a "not present" id?
    type ObjectId: Copy;
    type OutDist: Distribution<Output=(f32, Ray3, Self::ObjectId)>;

    fn intersect(&self, ray: Ray3, previous: Option<Self::ObjectId>) -> Option<Intersection<Self::OutDist>>;
}

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
