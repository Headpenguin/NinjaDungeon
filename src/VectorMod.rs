use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign};

#[derive(Copy, Clone, Debug)]
pub struct Vector(pub f32, pub f32);

impl Vector {
	pub fn fromPoints<I1, I2>(point1: I1, point2: I2) -> Vector where
	I1: Into<Vector>,
	I2: Into<Vector> {
		point2.into() - point1.into()
	}
	pub fn mag(&self) -> f32 {
		(self.0*self.0 + self.1*self.1).sqrt()
	}
	pub fn normalizeOrZero(&self) -> Vector {
		if self.0 == 0f32 && self.1 == 0f32 {Vector(0f32, 0f32)}
		else {*self / self.mag()}
	}
}

impl Add for Vector {
    type Output = Vector;
    fn add(self, other: Self) -> Self::Output {
        Vector(self.0 + other.0, self.1 + other.1)
    }
}

impl AddAssign for Vector {
	fn add_assign(&mut self, other: Self) {
		*self = *self + other;
	}
}

impl Sub for Vector {
	type Output = Vector;
	fn sub(self, other: Self) -> Self::Output {	
        Vector(self.0 - other.0, self.1 - other.1)
	}
}

impl SubAssign for Vector {
	fn sub_assign(&mut self, other: Self) {
		*self = *self - other;
	}
}

impl Mul<f32> for Vector {
	type Output = Vector;
	fn mul(self, other: f32) -> Self::Output {
		Vector(self.0 * other, self.1 * other)
	}
}

impl Div<f32> for Vector {
	type Output = Vector;
	fn div(self, other: f32) -> Self::Output {
		Vector(self.0 / other, self.1 / other)
	}
}

impl MulAssign<f32> for Vector {
	fn mul_assign(&mut self, other: f32) {
		*self = *self * other;
	}
}

impl DivAssign<f32> for Vector {
	fn div_assign(&mut self, other: f32) {
		*self = *self / other;
	}
}

impl From<Vector> for (i32, i32) {
	fn from(input: Vector) -> (i32, i32) {
		(input.0.round() as i32, input.1.round() as i32)
	}
}

impl From<Vector> for (f32, f32) {
	fn from(input: Vector) -> (f32, f32) {
		(input.0, input.1)
	}
}

impl From<(i32, i32)> for Vector {
	fn from(input: (i32, i32)) -> Vector {
		Vector(input.0 as f32, input.1 as f32)
	}
}

impl From<(f32, f32)> for Vector {
	fn from(input: (f32, f32)) -> Vector {
		Vector(input.0, input.1)
	}
}

