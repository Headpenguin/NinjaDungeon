use std::ops::{Deref, DerefMut};

pub struct Vec2d<T> (pub Vec<T>, usize);

impl<T> Vec2d<T> {
	pub fn new(v: Vec<T>, innerSize: usize) -> Vec2d<T> {
		Vec2d(v, innerSize)
	}
	pub fn index(&self, idx1: usize, idx2: usize) -> &T {
		&self.0[idx1 * self.1 + idx2]
	}
	pub fn indexMut(&mut self, idx1: usize, idx2: usize) -> &mut T {
		&mut self.0[idx1 * self.1 + idx2]
	}
}

impl<T> Deref for Vec2d<T> {
	type Target = Vec<T>;
	fn deref(&self) -> &Vec<T> {
		&self.0
	}
}

impl<T> DerefMut for Vec2d<T> {
	fn deref_mut(&mut self) -> &mut Vec<T> {
		&mut self.0
	}
}

