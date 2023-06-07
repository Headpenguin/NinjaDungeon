#![allow(non_snake_case)]
extern crate sdl2;
extern crate serde;
extern crate serde_json;
//extern crate BinaryFileIO;
//extern crate rlua;

use sdl2::{Sdl, VideoSubsystem, EventPump};
use sdl2::render::{Canvas, TextureCreator, Texture};
use sdl2::video::{WindowContext, Window};
use sdl2::event::Event;
use sdl2::hint;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};
use sdl2::ttf::{Sdl2TtfContext, Font, self};
use sdl2::keyboard::{TextInputUtil, Scancode};
use sdl2::mouse::MouseButton;

//use rlua::{Lua, FromLuaMulti, UserData, UserDataMethods};

//use BinaryFileIO::{load, dump};

use serde::{Serialize, Deserialize};

use serde_json::{Deserializer, Serializer};

use std::io::{self, Write};
use std::fs::File;
use std::i32;
use std::str::FromStr;
use std::hash::{Hasher, Hash};
use std::cell::UnsafeCell;
//use std::collections::BinaryHeap;

mod PlayerMod;
mod SpriteLoader;
mod MapMod;
mod Vec2dMod;
mod VectorMod;
mod IntHasher;
//mod ScriptingUtils;
mod EventProcessor;
mod Scheduling;
mod GameContextMod;
pub mod Entities;

pub use VectorMod::Vector;
pub use Vec2dMod::Vec2d;
pub use PlayerMod::Player;
//pub use ScriptingUtils::*;

pub use MapMod::{Map, InnerMap, Location, Tile, MAX_TILE_IDX, CollisionType, CollisionBounds, TileBuilder, TileBuilderSignals};
pub use GameContextMod::*;

use PlayerMod::SignalsBuilder;

//use Entities::{TypedID, Holder, Skeleton};
use Entities::{EntityBuilder, EntityBuilderSignals, EntityRenderer, MAX_ENTITY_IDX};
use Entities::Traits::EntityDyn;

pub use EventProcessor::PO;

use Scheduling::Scheduler;

/*const SCRIPTS_MISSING_MESSAGE: &str = "Lua scripts are missing, please refer to the website for further guidance (Error code 0001)\nThe following is diagnostic info";
const SYNTAX_ERROR: &str = "Lua scripts contain an error";
const MISSING_GLOBAL: &str = "Missing global";*/

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ID(u64);

const ID_MASK: u64 = 0xffffffffffffff00;

impl ID {
	pub fn new(id: u64, subID: u8) -> ID {
		ID((id << 8) + subID as u64)
	}
	pub fn sub(&self, subID: u8) -> ID {
		ID((self.0 & ID_MASK) + subID as u64)
	}
	pub fn empty() -> ID {
		ID(u64::MAX)
	}
	pub fn getID(&self) -> u64 {
		(self.0 & ID_MASK) >> 8
	}
	pub fn getSubID(&self) -> u8 {
		(self.0 & 0xff) as u8
	}
	pub fn isEmpty(&self) -> bool {
		self.0 == u64::MAX
	}
	pub fn mask(&self) -> ID {
		ID(self.0 & ID_MASK)
	}
}

pub struct GameManager {
	sdlContext: Sdl,
	videoSubsystem: VideoSubsystem,
	canvas: Canvas<Window>,
	events: EventPump,
	//luaContext: Lua,
	//scripts: Scripts,
//	scriptPlayerInputs: Option<Signals>,
	screenPos: Point,
	scheduler: Scheduler,
    //frameCounter: u64,
	quit: bool,
}


/*struct Scripts {
	initialInputs: LuaFunction,
}*/

impl GameManager {
	
	pub fn initialize(name: &'static str, width: u32, height: u32, color: Color) -> (GameManager, TextureCreator<WindowContext>) {
		
		let sdlContext = sdl2::init().unwrap();
		let videoSubsystem = sdlContext.video().unwrap();
		
		if !hint::set("SDL_RENDER_SCALE_QUALITY", "1") {
		
			eprintln!("Warning: Linear texture filtering may not be enabled.");
		
		}

		let window = videoSubsystem.window(name, width, height)
			.position_centered()
			.build()
			.unwrap();
		
		let mut canvas = window.into_canvas()
			.present_vsync()
			.build()
			.unwrap();
	
		let textureCreator = canvas.texture_creator();

		let events = sdlContext.event_pump().unwrap();
		
		canvas.set_draw_color(color);

		//let luaContext = Lua::new();

	/*	luaContext.context(|context| {
			context.globals().set("ninjaDungeonPushInputs", 
		});*/

		/*let mut scripts = LuaFunction::fromFunctionNames(&luaContext, fs::read_dir("Resources/Scripts/")
			.expect(SCRIPTS_MISSING_MESSAGE)
			.filter_map(|e| {
				let p = e.ok()?.path();
				if p.extension()? == "lua" {Some(p)}
				else {None}
			}),
			SCRIPT_FUNCTION_NAMES.iter().map(|s| *s)
		).expect(SYNTAX_ERROR);

		let scripts = Scripts {
			initialInputs: scripts.pop().expect(SCRIPTS_MISSING_MESSAGE).expect(MISSING_GLOBAL),
		};

		let (scriptPlayerInputs, quit, frameCounter) = (None, false, 0);*/
		let quit = false;
		let screenPos = Point::new(0, 0);

		(GameManager {sdlContext, videoSubsystem, canvas, events, /*luaContext, scripts, scriptPlayerInputs, frameCounter,*/ screenPos, quit, scheduler: Scheduler::new()}, textureCreator) 
	}
	
	#[inline(always)]
	pub fn mainLoop<'a, 'b>(&mut self, po: &'b mut UnsafeCell<PO<'a>>) -> bool {
		
		self.canvas.clear();
		
		let mut signals = SignalsBuilder::default();

		for event in self.events.poll_iter() {
			self.quit = Self::windowEvents(&event);
			signals.addEvent(&event);
		}
		
		let ctx = unsafe {po.get_mut().getCtxMut()};
		let player = ctx.holder.getMutTyped(ctx.getPlayerID()).unwrap();
		player.signal(signals.build(&self.events));
//		player.transition(&mut po.getCtx()map);
//		po.get_mut().transition();

		unsafe {
			if (&mut *po.get()).getCtxMut().getPlayerMut().transition((&mut *po.get()).getCtxMut()) {
				//println!("dgf");
				//po.get_mut().getCtxMut().resetCollisionLists();
				po.get_mut().getCtxMut().disableEntityCollisionFrame();
			}
		}
		

		unsafe {
			Scheduler::tick(po.get_mut().getCtxMut());
			self.scheduler.execute(po, |id| {(&mut *(&*po.get()).getCtx().getHolder().getEntityDyn(id).unwrap()).getData(&*po.get(), EventProcessor::Key::new());});
			po.get_mut().getCtxMut().resetCollisionLists();
			self.scheduler.execute(po, |id| (&mut *(&*po.get()).getCtx().getHolder().getEntityDyn(id).unwrap()).update(&mut *po.get()) );
		}
		unsafe {
			po.get_mut().getCtxMut().map.update();

			po.get_mut().doCommands();

			po.get_mut().getCtxMut().map.draw(&mut self.canvas, self.screenPos);
		}
		unsafe { self.scheduler.draw(po.get_mut().getCtx(), &mut self.canvas) ;}
		unsafe {po.get_mut().purge();}
		
		self.canvas.present();
		
		!self.quit
	}

	fn windowEvents(event: &Event) -> bool {
		match event {
			Event::Quit{..} => true,
			_ => false,
		}
	}
    //fn addInputs(&mut self, 
}

/*impl UserData for GameContext {
    fn add_methods<'lua, T>(methods: &mut T) where
    T: UserDataMethods<'lua, Self> {
        methods.add_method_mut("pushInputs", |context, this, args| {
            
        });
    }
}*/

pub struct EditorContext {
	sdlContext: Sdl,
	videoSubsystem: VideoSubsystem,
	canvas: Canvas<Window>,
	events: EventPump,
	textInput: TextInputUtil,
	color: Color,
	quit: bool,
    mapRes: (u32, u32),
    mapRect: Rect,
	newMapCoords: (u32, u32),
	newMapWidth: Option<u16>,
	currentTileId: u16,
//	currentTilePosition: (u16, u16),
	currentTile: Tile,
	previewTile: Tile,
	previewRect: Rect,
//	tileBuilder: TileBuilder,
	screenRect: Rect,
	screenPos: Rect,
	state: Vec<State>,
//	lock: bool,
	message: String,
	messageLen: usize,
	scheduler: Scheduler,
	globalEntities: bool,
	currentEntityId: u16,
//	entityBuilder: EntityBuilder,
}

impl EditorContext {
	pub fn new(width: u32, height: u32, name: &'static str, color: Color,) -> (TextureCreator<WindowContext>, Sdl2TtfContext, EditorContext) {

		let sdlContext = sdl2::init().unwrap();
		let videoSubsystem = sdlContext.video().unwrap();

		let textInput = videoSubsystem.text_input();
		textInput.stop();
		
		if !hint::set("SDL_RENDER_SCALE_QUALITY", "1") {
		
			eprintln!("Warning: Linear texture filtering may not be enabled.");
		
		}

		let window = videoSubsystem.window(name, width, height)
			.position_centered()
			.build()
			.unwrap();
		
		let canvas = window.into_canvas()
			.present_vsync()
			.build()
			.unwrap();

		let textureCreator = canvas.texture_creator();
		
		let ttfContext = ttf::init().unwrap();

		(textureCreator, ttfContext, EditorContext {
			canvas,
			events: sdlContext.event_pump().unwrap(),
			textInput,
			sdlContext,
			videoSubsystem,
			color,
			quit: false,
			currentTileId: 0,
//			currentTilePosition: (0, 0),
			currentTile: Tile::new(0, 0).unwrap(),
			previewTile: Tile::new(0, 0).unwrap(),
			screenRect: Rect::new(0, 0, width, height),
			screenPos: Rect::new(0, 0, width, height),
            mapRes: (136, 104),
			mapRect: Rect::new(0, 0, width, height),
			newMapCoords: (0, 0),
			newMapWidth: None,
			previewRect: Rect::new(0, height as i32 - 50, 50, 50),
//			tileBuilder: TileBuilder::new(0),
			state: vec![State::Idle],
//			lock: false,
			message: String::new(),
			messageLen: 0,
			scheduler: Scheduler::new(),
			globalEntities: false,
            currentEntityId: 0,
//			entityBuilder: EntityBuilder::new(u16::MAX, (0, 0)),
		})
	}

	pub fn mainLoop<'a>(&mut self, filename: &str, ctx: &mut GameContext<'a>, font: &Font, fontTexture: &mut Option<Texture<'a>>, idTexture: &mut Option<Texture<'a>>, entityRenderer: &EntityRenderer<'a>, textureCreator: &'a TextureCreator<WindowContext>) -> bool {
		self.canvas.set_draw_color(self.color);

		self.canvas.clear();
	
		for event in self.events.poll_iter() {match self.state.last().unwrap() {
			State::Idle => match event {
				Event::Quit {..} => self.quit = true,

				Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} 
				if (y as i64) < (self.screenRect.height() - 50) as i64 => {
					let (x, y) = (x + self.screenPos.x, y + self.screenPos.y);
					let currentTilePosition = ((x / 50) as u16, (y / 50) as u16);
					let tileBuilder = TileBuilder::new(self.currentTileId, currentTilePosition);
                    self.state.push(State::GetTile);
					self.state.push(State::AttemptBuild(tileBuilder));
					break;
				},
				Event::MouseButtonDown {mouse_btn: MouseButton::Right, x, y, ..}
                if (y as i64) < (self.screenRect.height() - 50) as i64 => {
					let (x, y) = (x + self.screenPos.x, y + self.screenPos.y);
					let currentTilePosition = ((x / 50) as u16, (y / 50) as u16);
                    ctx.getMapMut().changeTile(currentTilePosition, self.currentTile.clone());
               },
				Event::KeyDown{scancode: Some(Scancode::Left), ..} => {
					if self.currentTileId > 0 {
						self.currentTileId -= 1;
						self.previewTile = Tile::new(self.currentTileId, 0).unwrap();
					}
				},
				Event::KeyDown{scancode: Some(Scancode::Right), ..} => {
					if self.currentTileId < MAX_TILE_IDX {
						self.currentTileId += 1;
						self.previewTile = Tile::new(self.currentTileId, 0).unwrap();
					}
				},
				Event::KeyDown{scancode: Some(Scancode::S), ..} => {
					let mut ser = Serializer::new(File::create(filename).unwrap());
					unsafe {InnerGameContext::fromGameContext(ctx)}.serialize(&mut ser).unwrap();
					let mut f = ser.into_inner();
					f.flush().unwrap();
				},	
				Event::KeyDown{scancode: Some(Scancode::A), ..} => {
					ctx.getMapMut().decrementCurrentScreen();
					*idTexture = Some(createText(&ctx.getMap().getActiveScreenId().to_string(), textureCreator, font));
				},
				Event::KeyDown{scancode: Some(Scancode::D), ..} => {
					ctx.getMapMut().incrementCurrentScreen();
					*idTexture = Some(createText(&ctx.getMap().getActiveScreenId().to_string(), textureCreator, font));
				},
				Event::KeyDown{scancode: Some(Scancode::Escape), ..} => {
					self.state.push(State::ViewMap);
					*idTexture = None;
				},
				Event::KeyDown{scancode: Some(Scancode::M), ..} => {
					self.state.push(State::MoveScreen);
					*idTexture = None;
				},
				Event::KeyDown{scancode: Some(Scancode::X), ..} => {
					self.state.push(State::UserConfirmDelete);
					self.textInput.start();
					self.message = String::from("Are you sure you want to delete this map? (y/n): ");
					self.messageLen = self.message.len();
					*fontTexture = Some(createText(&self.message, textureCreator, font));
				},
				Event::KeyDown{scancode: Some(Scancode::E), ..} => {
					self.state.push(State::EntityPlacement);
				},

				Event::KeyDown {scancode: Some(Scancode::H), ..} => 
					self.screenPos.offset(self.screenRect.center().x.clamp(0, self.screenPos.x) * -1, 0),
				
				Event::KeyDown {scancode: Some(Scancode::J), ..} =>
					self.screenPos.offset(0, self.screenRect.center().y.clamp(0, self.screenPos.y) * -1),
			
				Event::KeyDown {scancode: Some(Scancode::K), ..} =>
					self.screenPos.offset(0, self.screenRect.center().y.min(ctx.getMap().getMaxScreenCoords().1 as i32 - self.screenPos.bottom()).max(0)),
				
				Event::KeyDown {scancode: Some(Scancode::L), ..} => 
					self.screenPos.offset(self.screenRect.center().x.min(ctx.getMap().getMaxScreenCoords().0 as i32 - self.screenPos.right()).max(0), 0),
				
				_ => (),
			},
			State::ViewMap => match event {
				Event::Quit {..} => self.quit = true,
                Event::KeyDown {scancode: Some(Scancode::Escape), ..} => {
                    self.state.pop();
					*idTexture = Some(createText(&ctx.getMap().getActiveScreenId().to_string(), textureCreator, font));
                }
				Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} => {
					if let Some(screen) = ctx.getMap().getScreenAtPosition(Point::new(x, y), self.mapRect, self.mapRes) {
						ctx.getMapMut().setCurrentScreen(screen);
						self.state.pop();
					}
					else {
						let (x, y) = MapMod::convertScreenCoordToTileCoord(self.mapRes, self.mapRect, Point::from((x, y))).into();
						self.newMapCoords = (x as u32, y as u32);
						self.message = String::from("Please enter the map width: ");
						self.messageLen = self.message.len();
						self.textInput.start();
						*fontTexture = Some(createText(&self.message, textureCreator, font));
						self.state.push(State::NewScreen);
					}
					*idTexture = Some(createText(&ctx.getMap().getActiveScreenId().to_string(), textureCreator, font));
                },
				Event::MouseWheel {y: 1, ..} => {
					if self.mapRes.0 > 1 && self.mapRes.1 > 1 {
						self.mapRes.0 /= 2;
						self.mapRes.1 /= 2;
					}
				},
				Event::MouseWheel {y: -1, ..} => {
					if self.mapRes.0 < i32::MAX as u32 && self.mapRes.1 < i32::MAX as u32 {
						self.mapRes.0 *= 2;
						self.mapRes.1 *= 2;
					}
				}
				Event::KeyDown {scancode: Some(Scancode::H), ..} => self.mapRect.offset(((self.mapRes.0 >> 1) as i32).clamp(0, self.mapRect.x()) * -1, 0),
				Event::KeyDown {scancode: Some(Scancode::J), ..} => self.mapRect.offset(0, ((self.mapRes.1 >> 1) as i32).clamp(0, self.mapRect.y()) * -1),
				Event::KeyDown {scancode: Some(Scancode::K), ..} => self.mapRect.offset(0, ((self.mapRes.1 >> 1) as i32).clamp(0, i32::MAX - self.mapRect.y())),
				Event::KeyDown {scancode: Some(Scancode::L), ..} => self.mapRect.offset(((self.mapRes.0 >> 1) as i32).clamp(0, i32::MAX - self.mapRect.x()), 0),
				_ => (),
			},
			State::MoveScreen => match event {
				Event::Quit {..} => self.quit = true,
                Event::KeyDown {scancode: Some(Scancode::Escape), ..} => {
                    self.state.pop();
					*idTexture = Some(createText(&ctx.getMap().getActiveScreenId().to_string(), textureCreator, font));
                },
				Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} => {
					let (x, y) = MapMod::convertScreenCoordToTileCoord(self.mapRes, self.mapRect, Point::from((x, y))).into();
					ctx.getMapMut().moveActiveScreen((x as u32, y as u32));
					self.state.pop();
					*idTexture = Some(createText(&ctx.getMap().getActiveScreenId().to_string(), textureCreator, font));
				},
				_ => (),
			},
			State::EntityPlacement => match event {
				Event::Quit {..} => self.quit = true,

				Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} 
				if (y as i64) < (self.screenRect.height() - 50) as i64 => {
					let (x, y) = (x + self.screenPos.x, y + self.screenPos.y);
					let entityPosition = ((x / 50) as u16, (y / 50) as u16);
					let entityBuilder = EntityBuilder::new(self.currentEntityId, entityPosition);
					let entityRect = entityBuilder.getEntityRect();
					if self.globalEntities {
						if let None = unsafe { ctx.getEntityAtPositionGlobal(entityRect)} {
							self.state.push(State::AttemptBuildEntity(entityBuilder));
							break;
						}
					} 
					else { 
						if let None = unsafe { ctx.getEntityAtPositionActiveScreen(entityRect)} {
							self.state.push(State::AttemptBuildEntity(entityBuilder));
							break;
						}
					}
				},
				Event::MouseButtonDown {mouse_btn: MouseButton::Right, x, y, ..}
				if (y as i64) < (self.screenRect.height() - 50) as i64 => {
					let (x, y) = ((x + self.screenPos.x) / 50 * 50, (y + self.screenPos.y) / 50 * 50);
					let clickRect = Rect::new(x, y, x as u32 + 50, y as u32 + 50);
					unsafe {
						if self.globalEntities {
							if let Some(id) = ctx.getEntityAtPositionGlobal(clickRect) {
								ctx.removeEntity(id);
							}
						}
						else {
							if let Some(id) = ctx.getEntityAtPositionActiveScreen(clickRect) {
								ctx.removeEntity(id);
							}
						}
					}

				},
				Event::KeyDown{scancode: Some(Scancode::Left), ..} => {
					if self.currentEntityId > 0 {
						self.currentEntityId -= 1;
					}
				},
				Event::KeyDown{scancode: Some(Scancode::Right), ..} => {
					if self.currentEntityId < MAX_ENTITY_IDX {
						self.currentEntityId += 1;
					}
				},
				Event::KeyDown{scancode: Some(Scancode::S), ..} => {
					let mut ser = Serializer::new(File::create(filename).unwrap());
					unsafe {InnerGameContext::fromGameContext(ctx)}.serialize(&mut ser).unwrap();
					let mut f = ser.into_inner();
					f.flush().unwrap();
				},	
				Event::KeyDown{scancode: Some(Scancode::A), ..} => {
					ctx.getMapMut().decrementCurrentScreen();
					*idTexture = Some(createText(&ctx.getMap().getActiveScreenId().to_string(), textureCreator, font));
				},
				Event::KeyDown{scancode: Some(Scancode::D), ..} => {
					ctx.getMapMut().incrementCurrentScreen();
					*idTexture = Some(createText(&ctx.getMap().getActiveScreenId().to_string(), textureCreator, font));
				},
				Event::KeyDown{scancode: Some(Scancode::Escape), ..} => {
					self.state.pop();
					*idTexture = None;
				},
				Event::KeyDown{scancode: Some(Scancode::M), ..} => {
					self.state.push(State::MoveScreen);
					*idTexture = None;
				},
				Event::KeyDown{scancode: Some(Scancode::X), ..} => {
					self.state.push(State::UserConfirmDelete);
					self.textInput.start();
					self.message = String::from("Are you sure you want to delete this map? (y/n): ");
					self.messageLen = self.message.len();
					*fontTexture = Some(createText(&self.message, textureCreator, font));
				},
				Event::KeyDown{scancode: Some(Scancode::E), ..} => {
					self.state.pop();
				},
				Event::KeyDown{scancode: Some(Scancode::G), ..} => {
					self.globalEntities = !self.globalEntities;
				},

				Event::KeyDown {scancode: Some(Scancode::H), ..} => 
					self.screenPos.offset(self.screenRect.center().x.clamp(0, self.screenPos.x) * -1, 0),
				
				Event::KeyDown {scancode: Some(Scancode::J), ..} =>
					self.screenPos.offset(0, self.screenRect.center().y.clamp(0, self.screenPos.y) * -1),
			
				Event::KeyDown {scancode: Some(Scancode::K), ..} =>
					self.screenPos.offset(0, self.screenRect.center().y.min(ctx.getMap().getMaxScreenCoords().1 as i32 - self.screenPos.bottom()).max(0)),
				
				Event::KeyDown {scancode: Some(Scancode::L), ..} => 
					self.screenPos.offset(self.screenRect.center().x.min(ctx.getMap().getMaxScreenCoords().0 as i32 - self.screenPos.right()).max(0), 0),
				
				_ => (),

			},
			_ => match (event, self.state.last().unwrap()) {
				(Event::Quit {..}, _) => self.quit = true,
				(Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..}, State::GetEntityID)
				if (y as i64) < (self.screenRect.height() - 50) as i64 => {
					let (x, y) = ((x + self.screenPos.x) / 50 * 50, (y + self.screenPos.y) / 50 * 50);
					let clickRect = Rect::new(x, y, x as u32 + 50, y as u32 + 50);
					unsafe {
						if self.globalEntities {
							if let Some(id) = ctx.getEntityAtPositionGlobal(clickRect) {
								self.state.pop();
								if let Some(State::AttemptBuildEntity(ref mut builder)) = self.state.last_mut() {
									builder.addLinkedID(id);
								}
							}
						}
						else {
							if let Some(id) = ctx.getEntityAtPositionActiveScreen(clickRect) {
								self.state.pop();
								if let Some(State::AttemptBuildEntity(ref mut builder)) = self.state.last_mut() {
									builder.addLinkedID(id);
								}
							}
						}
					}
					break;
				},
				(Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..}, State::GetTile)
				if (y as i64) < (self.screenRect.height() - 50) as i64 => {
					let (x, y) = (x + self.screenPos.x, y + self.screenPos.y);
					let currentTilePosition = ((x / 50) as u16, (y / 50) as u16);
					let tileBuilder = TileBuilder::new(self.currentTileId, currentTilePosition);
					self.state.push(State::AttemptBuild(tileBuilder));
                    *fontTexture = None;
					break;
				},
				(Event::MouseButtonDown {mouse_btn: MouseButton::Right, x, y, ..}, State::GetTile)
                if (y as i64) < (self.screenRect.height() - 50) as i64 => {
					let (x, y) = (x + self.screenPos.x, y + self.screenPos.y);
					let currentTilePosition = ((x / 50) as u16, (y / 50) as u16);
                    ctx.getMapMut().changeTile(currentTilePosition, self.currentTile.clone());
                    *fontTexture = None;
                },
				(Event::KeyDown{scancode: Some(Scancode::Left), ..}, State::GetTile) => {
					if self.currentTileId > 0 {
						self.currentTileId -= 1;
						self.previewTile = Tile::new(self.currentTileId, 0).unwrap();
					}
				},
				(Event::KeyDown{scancode: Some(Scancode::Right), ..}, State::GetTile) => {
					if self.currentTileId < MAX_TILE_IDX {
						self.currentTileId += 1;
						self.previewTile = Tile::new(self.currentTileId, 0).unwrap();
					}
				},
				(Event::KeyDown {scancode: Some(Scancode::Return), ..}, State::GetTile | State::GetEntityID) => {
					*fontTexture = None;
					self.state.pop();
					if let Some(State::AttemptBuildEntity(ref mut builder)) = self.state.last_mut() {
						builder.endList();
					}
					else {
						unreachable!();
					}
				},
				(Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..}, State::GetCoordinate) 
					if (y as i32) < (self.screenRect.height() - 50) as i32 => {
					self.state.pop();
					let location = ((x + self.screenPos.x) as u16 / 50u16, (y + self.screenPos.y) as u16 / 50u16);
					match self.state.last_mut().unwrap() {
                        State::AttemptBuild(ref mut builder) => {
                            builder.addLocation(location);
                        },
                        /*State::AttemptBuildEntity(ref mut builder) => {
                            builder.addLocation(location);
                        },*/
                        _ => unreachable!(),
                    };
                    
					*fontTexture = None;
				}
				(Event::KeyDown {scancode: Some(Scancode::Escape), ..}, _) => {
					self.state.pop();
					*fontTexture = None;
					*idTexture = Some(createText(&ctx.getMap().getActiveScreenId().to_string(), textureCreator, font));
				}
				(Event::KeyDown {scancode: Some(Scancode::Return), ..}, State::GetUserUsize) => {
					if let Ok(id) = usize::from_str(&self.message[self.messageLen..].trim()) {
						self.textInput.stop();
						*fontTexture = None;
                        self.state.pop();
                        match self.state.last_mut().unwrap() {
                            State::AttemptBuild(ref mut builder) => builder.addUsize(id),
                            //State::AttemptBuildEntity(ref mut builder) => builder.addUsize(id),
                            _ => panic!(),
                        }
					}
					else {
						self.message.truncate(self.messageLen);
						*fontTexture = Some(createText(&self.message, textureCreator, font));
					}
				},
				(Event::KeyDown {scancode: Some(Scancode::Return), ..}, State::UserConfirmDelete) => {
					if &"y" == &self.message[self.messageLen..].trim() {
						ctx.getMapMut().popActiveScreen();
					}
					self.textInput.stop();
					self.state.pop();
					*fontTexture = None;
					*idTexture = Some(createText(&ctx.getMap().getActiveScreenId().to_string(), textureCreator, font));
				},
				(Event::KeyDown {scancode: Some(Scancode::Return), ..}, State::NewScreen) => {
					if let Ok(dimension) = u16::from_str(&self.message[self.messageLen..].trim()) {
						if let Some(width) = self.newMapWidth  {
							ctx.getMapMut().addScreen(width, dimension, self.newMapCoords);
							self.newMapWidth = None;
							self.state.pop();
							self.textInput.stop();
							*fontTexture = None;
							*idTexture = Some(createText(&ctx.getMap().getActiveScreenId().to_string(), textureCreator, font));
						}
						else {
							self.newMapWidth = Some(dimension);
							self.message = String::from("Please enter the map height: ");
							self.messageLen = self.message.len();
							*fontTexture = Some(createText(&self.message, textureCreator, font));
						}
					}
					else {
						self.message.truncate(self.messageLen);
						*fontTexture = Some(createText(&self.message, textureCreator, font));
					}
				}
				(Event::TextInput {text, ..}, State::GetCoordinate) => (),
				(Event::TextInput {text, ..}, _) => {
					self.message.push_str(&text);
					*fontTexture = Some(createText(&self.message, textureCreator, font));
				},
				(Event::KeyDown {scancode: Some(Scancode::Backspace), ..}, _) if self.message.len() > self.messageLen => {
					self.message.pop();
					*fontTexture = Some(createText(&self.message, textureCreator, font));
				},
				_ => (),
			},
		}}

		match self.state.last_mut().unwrap() {
			State::AttemptBuild(ref mut builder) => {
                let signal = builder.build();
                self.build(signal, ctx.getMapMut(), font, fontTexture, textureCreator);
            },
			State::AttemptBuildEntity(ref mut builder) => {
                let signal = builder.build(textureCreator);
                self.buildEntity(signal, ctx, font, fontTexture, textureCreator);
            },
			_ => (),
		}

		match self.state.last().unwrap() {
			State::ViewMap | State::NewScreen | State::MoveScreen => {
				ctx.getMapMut().drawAll(&mut self.canvas, self.mapRes, self.mapRect);
			},
			State::EntityPlacement => {
				ctx.getMapMut().draw(&mut self.canvas, self.screenPos.top_left());
			},
			_ => {
				ctx.getMapMut().draw(&mut self.canvas, self.screenPos.top_left());
				ctx.getMapMut().renderTile(self.previewRect, &self.previewTile, &mut self.canvas);
			},
		}
		if let State::EntityPlacement | State::GetEntityID = self.state.last().unwrap() {
			if self.globalEntities {
				unsafe {self.scheduler.drawGlobal(&ctx, &mut self.canvas);}
			}
			else {
				unsafe {self.scheduler.drawNonGlobal(&ctx, &mut self.canvas);}
			}
			entityRenderer.render(&mut self.canvas, self.currentEntityId, self.previewRect);
		}
		if let Some(ref texture) = fontTexture {
			let q = texture.query();
			self.canvas.copy(texture, None, Some(Rect::from_center(self.screenRect.center(), q.width, q.height)));
		}
		if let Some(ref texture) = idTexture {
			let q = texture.query();
			self.canvas.copy(texture, None, Some(Rect::from_center((self.screenRect.right() - 25, self.screenRect.bottom() - 25), q.width, q.height)));
		}
		
		self.canvas.present();

		self.quit
	}
	fn build<'a>(&mut self, signal: TileBuilderSignals, map: &mut Map, font: &Font, fontTexture: &mut Option<Texture<'a>>, textureCreator: &'a TextureCreator<WindowContext>) {
		match signal {
			TileBuilderSignals::GetCoordinate(tmpMessage) => {
				self.state.push(State::GetCoordinate);
				*fontTexture = Some(createText(&self.message, textureCreator, font));
			}
			TileBuilderSignals::GetUserUsize(tmpMessage) => {
				self.state.push(State::GetUserUsize);
				self.textInput.start();
				self.message = String::from(tmpMessage);
				self.messageLen = tmpMessage.len() - 1;
				*fontTexture = Some(createText(&self.message, textureCreator, font));
			},
			TileBuilderSignals::Complete(tile, pos) => {
				let tilebuilder = if let Some(State::AttemptBuild(builder)) = self.state.pop() {builder} else {panic!()};
				self.currentTile = tile;
                if let State::GetTile = self.state.last().unwrap() {
                    self.state.pop();
                    match self.state.last_mut().unwrap() {
                        State::AttemptBuildEntity(ref mut builder) => builder.addTile(tilebuilder.cloneTile(&self.currentTile), pos),
                        State::Idle => map.changeTile(pos, tilebuilder.cloneTile(&self.currentTile)),
                        _ => unimplemented!(),
                    };
                }
                else {panic!()}
			},
			TileBuilderSignals::InvalidId => (),
		}	
	}
	fn buildEntity<'a>(&mut self, signal: EntityBuilderSignals<'a>, ctx: &mut GameContext<'a>, font: &Font, fontTexture: &mut Option<Texture<'a>>, creator: &'a TextureCreator<WindowContext>) {
		match signal {
			EntityBuilderSignals::Complete(Ok(entity)) if self.globalEntities => {
				if let Some(State::AttemptBuildEntity(builder)) = self.state.pop() {
    				builder.addEntityGlobal(ctx, entity);
                }
			},
			EntityBuilderSignals::Complete(Ok(entity)) => {
				if let Some(State::AttemptBuildEntity(builder)) = self.state.pop() {
    				builder.addEntityActiveScreen(ctx, entity);
                }
            },
			EntityBuilderSignals::Complete(Err(e)) => eprintln!("Entity could not be placed because of error (details below)\n{}", e),
            EntityBuilderSignals::GetTile(msg) => {
                *fontTexture = Some(createText(msg, creator, font));
                self.state.push(State::GetTile);
            },
			EntityBuilderSignals::GetEntity(msg) => {
				*fontTexture = Some(createText(msg, creator, font));
				self.state.push(State::GetEntityID);
			}
			EntityBuilderSignals::InvalidId => eprintln!("Entity could not be placed because of invalid entity id produced by editor"),
		}
	}

}

pub fn createText<'a>(message: &str, textureCreator: &'a TextureCreator<WindowContext>, font: &Font) -> Texture<'a> {
	font.render(message).shaded(Color::BLACK, Color::WHITE).unwrap().as_texture(textureCreator).unwrap()
}

pub fn loadCtx<'a>(filename: &str, creator: &'a TextureCreator<WindowContext>) -> io::Result<GameContext<'a>> {
	let mut deserializer = Deserializer::from_reader(File::open(filename)?);
	let ctx = InnerGameContext::deserialize(&mut deserializer)?;

	ctx.intoGameContext(creator)
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Direction {
	Up,
	Down,
	Left,
	Right,
}

//pub trait PlayerCollision {
  //  fn collidePlayer(&self, player: &mut Player);
//}

/*pub struct Entity {
	code: Codes,
	position: (i32, i32),
	direction: Direction,
}*/

enum State {
	GetUserUsize,
	GetCoordinate,
    GetTile,
	GetEntityID,
	UserConfirmDelete,
	ViewMap,
	NewScreen,
	MoveScreen,
	AttemptBuild(TileBuilder),
	AttemptBuildEntity(EntityBuilder),
	Idle,
	EntityPlacement,
}

const SCRIPT_FUNCTION_NAMES: &[&str] = &[
	"sendInputs",
];

