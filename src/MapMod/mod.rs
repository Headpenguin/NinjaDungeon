extern crate sdl2;

mod TileMod;
mod ScreenMod;

use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect;

use std::io;

pub use TileMod::*;
pub use ScreenMod::*;

use crate::SpriteLoader::Animations;

pub struct Map<'a> {
	screens: Vec<Screen>,
	activeScreen: usize,
	renderer: TileRenderer<'a>,
}

pub struct TileRenderer<'a> {
	animations: Animations<'a>,
}

impl<'a> Map<'a> {
	/*pub fn fromFile(filename: &str, tileset: &str, textureCreator: &'a TextureCreator) -> Map<'a> {
		
	}*/
	pub fn new(id: usize, activeScreen: usize, tileset: &str, textureCreator: &'a TextureCreator<WindowContext>) -> io::Result<Map<'a>> {
		let mut v = vec![];
		Ok(Map {
			screens: v,
			activeScreen: activeScreen,
			renderer: TileRenderer::new(id, tileset, textureCreator)?,
		})
	}
	pub fn draw(&self, canvas: &mut Canvas<Window>) {
		let mut rect = Rect::new(0, 0, 50, 50);
		for tile in self.screens[self.activeScreen]. {
			
		}
	}
}

impl<'a> TileRenderer<'a> {
	pub fn new(id: usize, tileset: &str, creator: &'a TextureCreator<WindowContext>) -> io::Result<TileRenderer<'a>> {
		Ok(TileRenderer {
			animations: Animations::new(tileset, TILESETS[id], creator)?,
		})
	}
}

const TILESETS: &'static [&'static [&'static str]] = &[
	&[
		"Ground",
		"Wall",
		"Gate",
	],
];

