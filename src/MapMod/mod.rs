mod TileMod;
mod ScreenMod;

use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::{Rect, Point};
use sdl2::pixels::Color;

use BinaryFileIO::BFStream::{ProvideReferencesDynamic, DynamicBinaryTranslator, ProvidePointersMutDynamic, DynamicTypedTranslator, SelfOwned};

use std::io;
use std::ptr::{addr_of_mut, self};
use std::collections::HashMap;
use std::alloc::{Layout, self};
use std::slice;

pub use TileMod::*;
pub use ScreenMod::*;

use crate::SpriteLoader::Animations;
use crate::IntHasher::IntHasher;

pub struct Map<'a> {
	screens: HashMap<usize, Screen, IntHasher>,
	lastActiveScreen: usize,
	activeScreen: usize,
	ioData: usize,
	nextId: usize,
	renderer: TileRenderer<'a>,
}

pub struct TileRenderer<'a> {
	animations: Animations<'a>,
}

impl<'a> Map<'a> {
	/*pub fn fromFile(filename: &str, tileset: &str, textureCreator: &'a TextureCreator) -> Map<'a> {
		
	}*/
	pub fn new(id: usize, tileset: &str, textureCreator: &'a TextureCreator<WindowContext>) -> io::Result<Map<'a>> {
		Ok(Map {
			screens: HashMap::with_hasher(IntHasher::new()),
			lastActiveScreen: 0,
			activeScreen: 0,
			nextId: 0,
			ioData: 0,
			renderer: TileRenderer::new(id, tileset, textureCreator)?,
		})
	}
	pub fn update(&mut self) {
		self.renderer.update();
	}
	pub fn draw(&mut self, canvas: &mut Canvas<Window>, topLeft: Point) {
		self.screens[&self.activeScreen].draw(&mut self.renderer, canvas, topLeft);
	}
	pub fn drawAll(&mut self, canvas: &mut Canvas<Window>, scale: (u32, u32), cameraRect: Rect) {
		let scale = (cameraRect.width() as f32 / scale.0 as f32, cameraRect.height() as f32 / scale.1 as f32);
		for screen in self.screens.values() {
			screen.iconDraw(&mut self.renderer, canvas, screen.generateIconRect(scale.0, scale.1, cameraRect.top_left()));
		}
		canvas.set_draw_color(Color::RED);
		canvas.draw_rect(self.screens[&self.activeScreen].generateIconRect(scale.0, scale.1, cameraRect.top_left()));
	}
	pub fn addScreen(&mut self, width: u16, height: u16, location: (u32, u32)) {
		self.screens.insert(self.nextId, Screen::new(width, height, location));
		self.lastActiveScreen = self.activeScreen;
		self.activeScreen = self.nextId;
		self.nextId+=1;
		self.ioData = self.screens.len();
	}
	pub fn popActiveScreen(&mut self) -> Option<Screen> {
		if self.screens.len() > 1 {
			let screen = self.screens.remove(&self.activeScreen).unwrap();
			if self.lastActiveScreen == self.activeScreen {
				self.activeScreen = *self.screens.keys().next().unwrap();
				self.lastActiveScreen = self.activeScreen;
			}
			else {
				self.activeScreen = self.lastActiveScreen;
			}
			self.ioData = self.screens.len();
			Some(screen)
		}
		else {None}
	}
	pub fn getScreen(&self, screen: usize) -> Option<&Screen> {
		self.screens.get(&screen)
	}
	pub fn getScreenAtPosition(&self, mut pos: Point, screenPos: Rect, res: (u32, u32)) -> Option<usize> {
		pos = convertScreenCoordToTileCoord(res, screenPos, pos);
		for (idx, screen) in self.screens.iter() {
		   if screen.containsPoint(pos) {
				return Some(*idx);
		   }
	   }
	   None
	}
	pub fn getMaxScreenCoords(&self) -> (u32, u32) {self.screens[&self.activeScreen].getMaxScreenCoords()}
	pub fn changeTile(&mut self, position: (u16, u16), replacement: Tile) {
		self.screens.get_mut(&self.activeScreen).unwrap().replaceTile(position, replacement);
	}
	pub fn renderTile(&mut self, position: Rect, tile: &Tile, canvas: &mut Canvas<Window>) {
		self.renderer.draw(tile, canvas, position);
	}
	pub fn incrementCurrentScreen(&mut self) {
		for screen in (self.activeScreen + 1)..self.nextId {
			if self.screens.contains_key(&screen) {
				self.lastActiveScreen = self.activeScreen;
				self.activeScreen = screen;
				break;
			}
		}
	}
	pub fn decrementCurrentScreen(&mut self) {
		for screen in (0..self.activeScreen).rev() {
			if self.screens.contains_key(&screen) {
				self.lastActiveScreen = self.activeScreen;
				self.activeScreen = screen;
				break;
			}
		}
	}
	pub fn setCurrentScreen(&mut self, screen: usize) -> Result<(), &'static str> {
		if self.screens.contains_key(&screen) {
			self.lastActiveScreen = self.activeScreen;
			self.activeScreen = screen;
			Ok(())
		}
		else {Err("Attempted to switch to invalid screen")}
	}
	pub fn getActiveScreenId(&self) -> usize {
		self.activeScreen
	}
	pub fn moveActiveScreen(&mut self, newPos: (u32, u32)) {
		self.screens.get_mut(&self.activeScreen).unwrap().moveToPosition(newPos);
	}
    pub fn transitionScreen(&mut self, hitbox: Rect) -> Option<Rect> {
        let activeScreen = &self.screens[&self.activeScreen];
		let (w, h) = activeScreen.getDimensions();
		let screenRect = Rect::new(0, 0, w as u32 * 50, h as u32 * 50);
		let center = hitbox.center();
		if !screenRect.contains_point(center) {
			let (screen, center) = match activeScreen.getScreen(center, self) {
				Some(data) => data,
				None => (self.activeScreen, center),
			};
			self.lastActiveScreen = self.activeScreen;
			self.activeScreen = screen;
			Some(Rect::from_center(center, hitbox.width(), hitbox.height()))
		}
		else {None}
    }
	#[inline(always)]
	pub fn calculateCollisionBounds(&self, hitbox: Rect) -> CollisionBounds {
		self.screens[&self.activeScreen].calculateCollisionBounds(hitbox)
	}
	pub fn collide(&'a self, bounds: &mut CollisionBounds) -> Option<((u16, u16), &'a Tile)> {
		self.screens[&self.activeScreen].collide(bounds)
	}
	pub unsafe fn createRenderer(&mut self, tileset: &str, textureCreator: &'a TextureCreator<WindowContext>) {
		addr_of_mut!(self.renderer).write(TileRenderer::new(0, tileset, textureCreator).unwrap());
	}
}

pub fn convertScreenCoordToTileCoord(res: (u32, u32), screenRect: Rect, point: Point) -> Point {
    Point::from((point.x * res.0 as i32 / screenRect.width() as i32, point.y * res.1 as i32 / screenRect.height() as i32)) + screenRect.top_left()
}

unsafe impl<'a> SelfOwned for Map<'a> {}

impl<'a> ProvideReferencesDynamic<'a> for Map<'a> {
	type Type = Map<'static>;
	fn provideReferencesDyn<T: DynamicBinaryTranslator<'a>>(&'a self, translator: &mut T) {
		translator.translateContained(&self.ioData);
		for (id, screen) in self.screens.iter() {
			translator.translateContained(id);
			translator.translateRaw(screen);
		}
		for screen in self.screens.values() {
			screen.provideReferencesDyn(translator);
		}
	}
}

impl<'a> ProvidePointersMutDynamic<'a> for Map<'a> {
	type Type = Map<'static>;
	unsafe fn providePointersMutDyn<T: DynamicTypedTranslator<'a>>(uninitialized: *mut Self, depth: usize, translator: &mut T) -> bool {
		match depth {
			0 => {
				translator.translateContained(&mut (*uninitialized).ioData);
				false
			},
			1 => {
				let len = (*uninitialized).ioData;
				addr_of_mut!((*uninitialized).screens).write(HashMap::with_capacity_and_hasher(len, IntHasher::new()));
//				(*uninitialized).lastActiveScreen = (*uninitialized).activeScreen;
				let layout = Layout::array::<(usize, Screen)>(len + 1).unwrap();
				let memory = alloc::alloc(layout);
				assert_ne!(memory, ptr::null_mut());
				(*uninitialized).ioData = memory as usize;
				let memory = memory as *mut (usize, Screen);
				let entries = slice::from_raw_parts_mut(memory.add(1), len);
				(*memory).0 = len;
				for (id, screen) in entries.iter_mut() {
					translator.translateContained(id);
					translator.translateRaw(screen);
				}
				false
			},
			depth => {
				let ptr = (*uninitialized).ioData as *mut (usize, Screen);
				let len = (*ptr).0;
				let mut flag = true;
				{
					let entries = slice::from_raw_parts_mut(ptr.add(1), len);
					for (_, screen) in entries.iter_mut() {
						let result = Screen::providePointersMutDyn(screen, depth - 2, translator);
						flag &= result;
					}
				}
				if flag {
					{
						let entries = slice::from_raw_parts_mut(ptr.add(1), len);
						(*uninitialized).screens.extend(entries.iter().map(|e| ptr::read(e)));
					}
					let layout = Layout::array::<usize>(len + 1).unwrap();
					alloc::dealloc(ptr as *mut u8, layout);
				}
				flag
			},
		}
/*		if depth == 0 {
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
		}*/
	}
}

impl<'a> TileRenderer<'a> {
	pub fn new(id: usize, tileset: &str, creator: &'a TextureCreator<WindowContext>) -> io::Result<TileRenderer<'a>> {
		Ok(TileRenderer {
			animations: Animations::new(tileset, TILESETS[id], creator)?,
		})
	}
	pub fn update(&mut self) {
		self.animations.update();
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

