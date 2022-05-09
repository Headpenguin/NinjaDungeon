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
		Ok(Map {
			screens: vec![],
			activeScreen: activeScreen,
			renderer: TileRenderer::new(id, tileset, textureCreator)?,
		})
	}
	pub fn draw(&mut self, canvas: &mut Canvas<Window>) {
		self.screens[self.activeScreen].draw(&mut self.renderer, canvas);
	}
	pub fn addScreen(&mut self, width: u16, height: u16, location: Location) {
		self.screens.push(Screen::new(width, height, location));
	}
	pub fn changeTile(&mut self, position: (u16, u16), replacement: Tile) {
		self.screens[self.activeScreen].replaceTile(position, replacement);
	}
	pub fn renderTile(&mut self, position: Rect, tile: &Tile, canvas: &mut Canvas<Window>) {
		self.renderer.draw(tile, canvas, position);
	}
}

impl<'a> TileRenderer<'a> {
	pub fn new(id: usize, tileset: &str, creator: &'a TextureCreator<WindowContext>) -> io::Result<TileRenderer<'a>> {
		Ok(TileRenderer {
			animations: Animations::new(tileset, TILESETS[id], creator)?,
		})
	}
	// Make this better pls
	pub fn draw(&mut self, tile: &Tile, canvas: &mut Canvas<Window>, position: Rect) {
		self.animations.changeAnimation(tile.getId() as usize).unwrap();
		self.animations.drawNextFrame(canvas, position);
	}
}

const TILESETS: &'static [&'static [&'static str]] = &[
	&[
		"Ground",
		"Wall",
		"Gate",
	],
];

