//use BinaryFileIO::BFStream::{ProvideReferencesDynamic, DynamicBinaryTranslator, ProvidePointersMutDynamic, DynamicTypedTranslator};
//use BinaryFileIO::BinaryDataContainer;

use serde::{Serialize, Deserialize};

use std::ops::{Deref, DerefMut};
//use std::ptr::addr_of_mut;

#[derive(Serialize, Deserialize, Clone)]
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
	pub fn get(&self, idx1: usize, idx2: usize) -> Option<&T> {
		self.0.get(idx1 * self.1 + idx2)
	}
	pub fn getMut(&mut self, idx1: usize, idx2: usize) -> Option<&mut T> {
		self.0.get_mut(idx1*self.1 + idx2)
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
/*
impl<'a, Ty> ProvideReferencesDynamic<'a> for Vec2d<Ty> where Ty: ProvideReferencesDynamic<'a> {
	type Type = ();
	fn provideReferencesDyn<T: DynamicBinaryTranslator<'a>>(&'a self, translator: &mut T) {
		unsafe{translator.translateSlice(self.0.as_slice());}
	}
}

impl<'a, Ty> ProvidePointersMutDynamic<'a> for Vec2d<Ty> where Ty: ProvidePointersMutDynamic<'a> {
	type Type = ();
	unsafe fn providePointersMutDyn<T: DynamicTypedTranslator<'a>>(uninitialized: *mut Self, depth: usize, translator: &mut T) -> bool {
		if depth == 0 {
			let size = translator.getSliceSize().unwrap();
			let mut v = Vec::with_capacity(size);
			let ptr = v.as_mut_ptr();
			let translatedPtr: *mut [Ty] = BinaryDataContainer::reinterpretAllocatedToSlice(ptr as *mut u8, size);
			translator.translateRawSlice(translatedPtr);
			v.set_len(size);
			addr_of_mut!((*uninitialized).0).write(v);
			false
		}
		else {
			translator.translateSlice(depth - 1, (*uninitialized).0.as_mut_slice())
		}
	}
}
*/
