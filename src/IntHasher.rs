use std::hash::{Hasher, BuildHasher};

#[derive(Default)]
pub struct IntHasher (u64);

impl IntHasher {
	pub fn new() -> IntHasher {IntHasher(0)}
}

impl Hasher for IntHasher {
	fn write(&mut self, _: &[u8]) {panic!("IntHasher only hashes usizes");}
	
	fn write_usize(&mut self, id: usize) {
		self.0 = id as u64;
	}
	fn finish(&self) -> u64 {self.0}
}

impl BuildHasher for IntHasher {
	type Hasher = Self;
	fn build_hasher(&self) -> Self {
		IntHasher(0)
	}
}

