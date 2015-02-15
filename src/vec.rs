use std::ops::{Add, Sub, Mul};

#[derive(Copy)]
pub struct Vec3 {
    vals: [f32; 3]
}

impl Vec3 {
	pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
		Vec3 {
			vals: [x, y, z]
		}
	}

    pub fn dot(self, other: Vec3) -> f32 {
        self.vals[0] * other.vals[0] + self.vals[1] * other.vals[1] + self.vals[2] * other.vals[2]
    }

    pub fn mag2(self) -> f32 {
        self.dot(self)
    }
}

impl Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            vals: [
                self.vals[0] + other.vals[0],
                self.vals[1] + other.vals[1],
                self.vals[2] + other.vals[2]
            ]
        }
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            vals: [
                self.vals[0] - other.vals[0],
                self.vals[1] - other.vals[1],
                self.vals[2] - other.vals[2]
            ]
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, other: f32) -> Vec3 {
        Vec3 {
            vals: [
                self.vals[0] * other,
                self.vals[1] * other,
                self.vals[2] * other
            ]
        }
    }
}
