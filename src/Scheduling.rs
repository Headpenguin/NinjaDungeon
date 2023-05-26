use std::iter::Iterator;
use crate::ID;
pub struct Iter {}
impl Iterator for Iter {
	type Item = ID;
	fn next(&mut self) -> Option<Self::Item> {panic!()}
}
pub struct Scheduler{}
impl Scheduler {
	pub fn needsExecution(&self) -> Iter {panic!();}
}
