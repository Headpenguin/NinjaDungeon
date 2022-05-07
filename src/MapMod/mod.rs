mod TileMod;
mod ScreenMod;

use crate::Vec2d;

pub use TileMod::*;
pub use ScreenMod::*;

pub struct Map {
	screens: Vec2d<Screen>,
}

