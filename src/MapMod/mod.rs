mod TileMod;
mod ScreenMod;

use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect;

use BinaryFileIO::BFStream::{ProvideReferencesDynamic, DynamicBinaryTranslator, ProvidePointersMutDynamic, DynamicTypedTranslator, SelfOwned};
use BinaryFileIO::BinaryDataContainer;

use std::io;
use std::ptr::addr_of_mut;

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
		self.activeScreen = self.screens.len() - 1;
	}
	pub fn changeTile(&mut self, position: (u16, u16), replacement: Tile) {
		self.screens[self.activeScreen].replaceTile(position, replacement);
	}
	pub fn renderTile(&mut self, position: Rect, tile: &Tile, canvas: &mut Canvas<Window>) {
		self.renderer.draw(tile, canvas, position);
	}
	pub fn incrementCurrentScreen(&mut self) {
		if self.activeScreen + 1 < self.screens.len() {self.activeScreen+=1;}
	}
	pub fn decrementCurrentScreen(&mut self) {
		if self.activeScreen > 0 {self.activeScreen-=1;}
	}
	pub unsafe fn createRenderer(&mut self, tileset: &str, textureCreator: &'a TextureCreator<WindowContext>) {
		addr_of_mut!(self.renderer).write(TileRenderer::new(0, tileset, textureCreator).unwrap());
	}
}

unsafe impl<'a> SelfOwned for Map<'a> {}

impl<'a> ProvideReferencesDynamic<'a> for Map<'a> {
	type Type = Map<'static>;
	fn provideReferencesDyn<T: DynamicBinaryTranslator<'a>>(&'a self, translator: &mut T) {
		unsafe{translator.translateSlice(self.screens.as_slice())};
	}
}

impl<'a> ProvidePointersMutDynamic<'a> for Map<'a> {
	type Type = Map<'static>;
	unsafe fn providePointersMutDyn<T: DynamicTypedTranslator<'a>>(uninitialized: *mut Self, depth: usize, translator: &mut T) -> bool {
		if depth == 0 {
			let size = translator.getSliceSize().unwrap();
			let mut v = Vec::with_capacity(size);
			let ptr = v.as_mut_ptr();
			let translatedPtr: *mut [Screen] = BinaryDataContainer::reinterpretAllocatedToSlice(ptr as *mut u8, size);
			translator.translateRawSlice(translatedPtr);
			v.set_len(size);
			addr_of_mut!((*uninitialized).screens).write(v);
			false
		}
		else {
			translator.translateSlice(depth - 1, (*uninitialized).screens.as_mut_slice())
		}
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

