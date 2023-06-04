use std::hash::{Hasher, BuildHasher};
use std::default::Default;

#[derive(Default, Clone)]
pub struct UInt64Hasher (u64);

impl Hasher for UInt64Hasher {
	fn write(&mut self, _: &[u8]) {panic!("UInt64Hasher only hashes u64");}
	
	fn write_u64(&mut self, id: u64) {
		self.0 = id;
	}
	fn finish(&self) -> u64 {self.0}
}

#[derive(Default, Clone)]
pub struct USizeHasher (usize);
impl USizeHasher {
	pub fn new() -> USizeHasher {
		USizeHasher(0)
	}
}
impl Hasher for USizeHasher {
	fn write(&mut self, _: &[u8]) {panic!("USizeHasher only hashes usize");}
	
	fn write_usize(&mut self, data: usize) {
		self.0 = data;
	}
	fn finish(&self) -> u64 {self.0 as u64}
}

impl BuildHasher for UInt64Hasher {
	type Hasher = Self;
	fn build_hasher(&self) -> Self {
		UInt64Hasher(0)
	}
}
impl BuildHasher for USizeHasher {
	type Hasher = Self;
	fn build_hasher(&self) -> Self {
		USizeHasher(0)
	}
}

