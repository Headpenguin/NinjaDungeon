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

use serde::Serialize;

use serde_json::Serializer;

use std::fs::File;
use std::io::Write;
use std::str::FromStr;

use crate::Scheduling::Scheduler;

use crate::MapMod::{TileBuilder, TileBuilderSignals, Tile, Map, self};
use crate::{GameContext, MAX_TILE_IDX, MAX_COLLISION_IDX, InnerGameContext};
use crate::Entities::{EntityBuilder, EntityBuilderSignals, EntityRenderer, MAX_ENTITY_IDX};

pub struct EditorContext {
	sdlContext: Sdl,
	videoSubsystem: VideoSubsystem,
	canvas: Canvas<Window>,
	textInput: TextInputUtil,
	color: Color,
	quit: bool,
    mapRes: (u32, u32),
    mapRect: Rect,
	newMapCoords: (u32, u32),
	newMapWidth: Option<u16>,
	currentTileId: u16,
    currentCollision: usize,
	currentTile: Tile,
	previewTile: Tile,
	previewRect: Rect,
	screenRect: Rect,
	screenPos: Rect,
	state: Vec<State>,
	message: String,
	messageLen: usize,
	scheduler: Scheduler,
	globalEntities: bool,
	currentEntityId: u16,
}

pub struct EditorContextDeps<'tex, 'ttf, 'ctx, 'filename, 'idTex, 'collisionTex, 'fontTex, 'font, 'entTex> {
    pub filename: &'filename str,
    pub ctx: &'ctx mut GameContext<'tex>,
    pub font: &'font Font<'ttf, 'static>,
    pub fontTexture: &'fontTex mut Option<Texture<'tex>>,
    pub idTexture: &'idTex mut Option<Texture<'tex>>,
    pub collisionTextures: &'collisionTex [Texture<'tex>],
    pub entityRenderer: &'entTex EntityRenderer<'tex>,
    pub textureCreator: &'tex TextureCreator<WindowContext>,
}

impl EditorContext {
	pub fn new(width: u32, height: u32, name: &'static str, color: Color,) -> (TextureCreator<WindowContext>, Sdl2TtfContext, EventPump, EditorContext) {

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

		(textureCreator, ttfContext, sdlContext.event_pump().unwrap(), EditorContext {
			canvas,
			textInput,
			sdlContext,
			videoSubsystem,
			color,
			quit: false,
			currentTileId: 0,
            currentCollision: 0,
			currentTile: Tile::default(),
			previewTile: Tile::default(),
			screenRect: Rect::new(0, 0, width, height),
			screenPos: Rect::new(0, 0, width, height),
            mapRes: (136, 104),
			mapRect: Rect::new(0, 0, width, height),
			newMapCoords: (0, 0),
			newMapWidth: None,
			previewRect: Rect::new(0, height as i32 - 50, 50, 50),
			state: vec![State::ViewMap, State::Idle],
			message: String::new(),
			messageLen: 0,
			scheduler: Scheduler::new(),
			globalEntities: false,
            currentEntityId: 0,
		})
    }	

	pub fn mainLoop<'a>(&mut self, events: &mut EventPump, deps: &mut EditorContextDeps) -> bool {
        self.canvas.set_draw_color(self.color);

		self.canvas.clear();
	
		for event in events.poll_iter() {match self.state.last().unwrap() {
			State::Idle => self.doIdleEvents(event, deps),
			State::ViewMap => self.doViewMapEvents(event, deps),
            State::MoveScreen => self.doMoveScreenEvents(event, deps),
            State::EntityPlacement => self.doEntityPlacementEvents(event, deps),
            State::MakeEntityInactive => self.doRestrictedEntityPlacementEvents(event, deps),
            _ => match (event, self.state.last().unwrap()) {
				(Event::Quit {..}, _) => self.quit = true,
				(Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..}, State::GetEntityID)
				if (y as i64) < (self.screenRect.height() - 50) as i64 => {
					let (x, y) = ((x + self.screenPos.x) / 50 * 50, (y + self.screenPos.y) / 50 * 50);
					let clickRect = Rect::new(x, y, x as u32 + 50, y as u32 + 50);
					unsafe {
						let id = if self.globalEntities {deps.ctx.getEntityAtPositionGlobal(clickRect)}
						else {deps.ctx.getEntityAtPositionActiveScreen(clickRect)};
						if let Some(id) = id {
							self.state.pop();
							if let Some(State::AttemptBuildEntity(ref mut builder)) = self.state.last_mut() {
								builder.addLinkedID(id);
							}
						}
					}
					break;
				},
				(Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..}, State::GetTile)
				if (y as i64) < (self.screenRect.height() - 50) as i64 => {
					let currentTilePosition = convertToTilePos(x + self.screenPos.x, y + self.screenPos.y);
					let tileBuilder = TileBuilder::new(self.currentTileId, self.currentCollision, currentTilePosition);
					self.state.push(State::AttemptBuild(tileBuilder));
                    *deps.fontTexture = None;
					break;
				},
				(Event::MouseButtonDown {mouse_btn: MouseButton::Right, x, y, ..}, State::GetTile)
                if (y as i64) < (self.screenRect.height() - 50) as i64 => {
					let currentTilePosition = convertToTilePos(x + self.screenPos.x, y + self.screenPos.y);
                    //deps.ctx.getMapMut().changeTile(currentTilePosition, self.currentTile.clone());
					self.state.push(State::AttemptBuild(TileBuilder::fromTile(&self.currentTile, currentTilePosition)));
                    *deps.fontTexture = None;
                },
				(Event::KeyDown{scancode: Some(Scancode::Left), ..}, State::GetTile) => {
                    self.incTile(-1);
				},
				(Event::KeyDown{scancode: Some(Scancode::Right), ..}, State::GetTile) => {
                    self.incTile(1);
				},
                (Event::KeyDown{scancode: Some(Scancode::Up), ..}, State::GetTile) => {
                    self.incCollision(1);
                },
                (Event::KeyDown{scancode: Some(Scancode::Down), ..}, State::GetTile) => {
                    self.incCollision(-1);
                },
				(Event::KeyDown {scancode: Some(Scancode::Return), ..}, State::GetTile | State::GetEntityID) => self.endList(deps),
				(Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..}, State::GetCoordinate) 
					if (y as i32) < (self.screenRect.height() - 50) as i32 => {
					self.state.pop();
					let location = ((x + self.screenPos.x) as u16 / 50u16, (y + self.screenPos.y) as u16 / 50u16);
					match self.state.last_mut().unwrap() {
                        State::AttemptBuild(ref mut builder) => {
                            builder.addLocation(location);
                        },
						State::AttemptBuildEntity(ref mut Builder) => {
							unimplemented!();
						}
                        _ => unreachable!(),
                    };
                    
					*deps.fontTexture = None;
				}
				(Event::KeyDown {scancode: Some(Scancode::Escape), ..}, _) => {
					self.state.pop();
                    if let Some(State::AttemptBuildEntity(..)) | Some(State::AttemptBuild(..)) = self.state.last() {
                        self.state.pop();
                    }
					*deps.fontTexture = None;
	//				*deps.idTexture = Some(createText(&deps.ctx.getMap().getActiveScreenId().to_string(), deps.textureCreator, deps.font));
				}
				(Event::KeyDown {scancode: Some(Scancode::Return), ..}, State::GetUserUsize) => {
					if let Ok(id) = usize::from_str(&self.message[self.messageLen..].trim()) {
						self.textInput.stop();
						*deps.fontTexture = None;
                        self.state.pop();
                        match self.state.last_mut().unwrap() {
                            State::AttemptBuild(ref mut builder) => builder.addUsize(id),
                            State::AttemptBuildEntity(ref mut builder) => unimplemented!(),
                            _ => unreachable!(),
                        }
					}
					else {
						self.message.truncate(self.messageLen);
						*deps.fontTexture = Some(createText(&self.message, deps.textureCreator, deps.font));
					}
				},
				(Event::KeyDown {scancode: Some(Scancode::Return), ..}, State::UserConfirmDelete) => {
					if &"y" == &self.message[self.messageLen..].trim() {
						deps.ctx.getMapMut().popActiveScreen();
					}
					self.textInput.stop();
					self.state.pop();
					*deps.fontTexture = None;
					*deps.idTexture = Some(createText(&deps.ctx.getMap().getActiveScreenId().to_string(), deps.textureCreator, deps.font));
				},
				(Event::KeyDown {scancode: Some(Scancode::Return), ..}, State::NewScreen) => {
					if let Ok(dimension) = u16::from_str(&self.message[self.messageLen..].trim()) {
						if let Some(width) = self.newMapWidth  {
							deps.ctx.getMapMut().addScreen(width, dimension, self.newMapCoords);
							self.newMapWidth = None;
							self.state.pop();
							self.textInput.stop();
							*deps.fontTexture = None;
							*deps.idTexture = Some(createText(&deps.ctx.getMap().getActiveScreenId().to_string(), deps.textureCreator, deps.font));
						}
						else {
							self.newMapWidth = Some(dimension);
							self.message = String::from("Please enter the map height: ");
							self.messageLen = self.message.len();
							*deps.fontTexture = Some(createText(&self.message, deps.textureCreator, deps.font));
						}
					}
					else {
						self.message.truncate(self.messageLen);
						*deps.fontTexture = Some(createText(&self.message, deps.textureCreator, deps.font));
					}
				}
				(Event::TextInput {..}, State::GetCoordinate) => (),
				(Event::TextInput {text, ..}, _) => {
					self.message.push_str(&text);
					*deps.fontTexture = Some(createText(&self.message, deps.textureCreator, deps.font));
				},
				(Event::KeyDown {scancode: Some(Scancode::Backspace), ..}, _) if self.message.len() > self.messageLen => {
					self.message.pop();
					*deps.fontTexture = Some(createText(&self.message, deps.textureCreator, deps.font));
				},
				_ => (),
			},
		}}

		match self.state.last_mut().unwrap() {
			State::AttemptBuild(ref mut builder) => {
                let signal = builder.build();
                self.build(signal, deps);
            },
			State::AttemptBuildEntity(ref mut builder) => {
                let signal = builder.build(deps.textureCreator);
                self.buildEntity(signal, deps);
            },
			_ => (),
		}

		match self.state.last().unwrap() {
			State::ViewMap | State::NewScreen | State::MoveScreen => {
				deps.ctx.getMapMut().drawAll(&mut self.canvas, self.mapRes, self.mapRect);
			},
			State::EntityPlacement => {
				deps.ctx.getMapMut().draw(&mut self.canvas, self.screenPos.top_left());
			},
			_ => {
				deps.ctx.getMapMut().draw(&mut self.canvas, self.screenPos.top_left());
				deps.ctx.getMapMut().renderTile(self.previewRect, &self.previewTile, &mut self.canvas);
                let q = deps.collisionTextures[self.currentCollision].query();
                self.canvas.copy(&deps.collisionTextures[self.currentCollision], None, Some(Rect::new(self.previewRect.x() + 100, self.previewRect.y(), q.width, q.height)));
			},
		}
		if let State::EntityPlacement | State::GetEntityID | State::MakeEntityInactive = self.state.last().unwrap() {
			if self.globalEntities {
				unsafe {self.scheduler.drawGlobal(&deps.ctx, &mut self.canvas);}
			}
			else {
				unsafe {self.scheduler.drawNonGlobal(&deps.ctx, &mut self.canvas);}
			}
			deps.entityRenderer.render(&mut self.canvas, self.currentEntityId, self.previewRect);
		}
		if let Some(ref texture) = deps.fontTexture {
			let q = texture.query();
			self.canvas.copy(texture, None, Some(Rect::from_center(self.screenRect.center(), q.width, q.height)));
		}
		if let Some(ref texture) = deps.idTexture {
			let q = texture.query();
			self.canvas.copy(texture, None, Some(Rect::from_center((self.screenRect.right() - 25, self.screenRect.bottom() - 25), q.width, q.height)));
		}
		
		self.canvas.present();

		self.quit
	}
    fn doIdleEvents(&mut self, event: Event, deps: &mut EditorContextDeps) {
        match event {
            Event::Quit {..} => self.quit = true,

            Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} 
            if (y as i64) < (self.screenRect.height() - 50) as i64 => {
                let currentTilePosition = convertToTilePos(x + self.screenPos.x, y + self.screenPos.y);
                let tileBuilder = TileBuilder::new(self.currentTileId, self.currentCollision, currentTilePosition);
                self.state.push(State::GetTile);
                self.state.push(State::AttemptBuild(tileBuilder));
            },
            Event::MouseButtonDown {mouse_btn: MouseButton::Right, x, y, ..}
            if (y as i64) < (self.screenRect.height() - 50) as i64 => {
                let currentTilePosition = convertToTilePos(x + self.screenPos.x, y + self.screenPos.y);
                deps.ctx.getMapMut().changeTile(currentTilePosition, self.currentTile.clone());
           },
            Event::KeyDown{scancode: Some(Scancode::Left), ..} => {
                self.incTile(-1);
            },
            Event::KeyDown{scancode: Some(Scancode::Right), ..} => {
                self.incTile(1);
            },
            Event::KeyDown{scancode: Some(Scancode::Up), ..} => {
                self.incCollision(1);
            },
            Event::KeyDown{scancode: Some(Scancode::Down), ..} => {
                self.incCollision(-1);
            },
            Event::KeyDown{scancode: Some(Scancode::E), ..} => {
                self.state.push(State::EntityPlacement);
            },
            _ => self.matchCommon(event, deps),
        }
    }
    fn doViewMapEvents(&mut self, event: Event, deps: &mut EditorContextDeps) {
        match event {
            Event::Quit {..} => self.quit = true,
            Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} => {
                if let Some(screen) = deps.ctx.getMap().getScreenAtPosition(Point::new(x, y), self.mapRect, self.mapRes) {
                    deps.ctx.getMapMut().setCurrentScreen(screen);
                    self.state.push(State::Idle);
                }
                else {
                    let (x, y) = MapMod::convertScreenCoordToTileCoord(self.mapRes, self.mapRect, Point::from((x, y))).into();
                    self.newMapCoords = (x as u32, y as u32);
                    self.message = String::from("Please enter the map width: ");
                    self.messageLen = self.message.len();
                    self.textInput.start();
                    *deps.fontTexture = Some(createText(&self.message, deps.textureCreator, deps.font));
                    self.state.push(State::NewScreen);
                }
                *deps.idTexture = Some(createText(&deps.ctx.getMap().getActiveScreenId().to_string(), deps.textureCreator, deps.font));
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
            },
            Event::KeyDown {scancode: Some(Scancode::H), ..} => self.mapRect.offset(((self.mapRes.0 >> 1) as i32).clamp(0, self.mapRect.x()) * -1, 0),
            Event::KeyDown {scancode: Some(Scancode::J), ..} => self.mapRect.offset(0, ((self.mapRes.1 >> 1) as i32).clamp(0, self.mapRect.y()) * -1),
            Event::KeyDown {scancode: Some(Scancode::K), ..} => self.mapRect.offset(0, ((self.mapRes.1 >> 1) as i32).clamp(0, i32::MAX - self.mapRect.y())),
            Event::KeyDown {scancode: Some(Scancode::L), ..} => self.mapRect.offset(((self.mapRes.0 >> 1) as i32).clamp(0, i32::MAX - self.mapRect.x()), 0),
            Event::KeyDown {scancode: Some(Scancode::Escape), ..} => (),
            _ => (),
        }
    }
    fn doMoveScreenEvents(&mut self, event: Event, deps: &mut EditorContextDeps) {
        match event {
            Event::Quit {..} => self.quit = true,
            Event::KeyDown {scancode: Some(Scancode::Escape), ..} => {
                self.state.pop();
                *deps.idTexture = Some(createText(&deps.ctx.getMap().getActiveScreenId().to_string(), deps.textureCreator, deps.font));
            },
            Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} => {
                let (x, y) = MapMod::convertScreenCoordToTileCoord(self.mapRes, self.mapRect, Point::from((x, y))).into();
                deps.ctx.getMapMut().moveActiveScreen((x as u32, y as u32));
                self.state.pop();
                *deps.idTexture = Some(createText(&deps.ctx.getMap().getActiveScreenId().to_string(), deps.textureCreator, deps.font));
            },
            _ => (),
        }
    }
    fn doEntityPlacementEvents(&mut self, event: Event, deps: &mut EditorContextDeps) {
        match event {
            Event::Quit {..} => self.quit = true,

            Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} 
            if (y as i64) < (self.screenRect.height() - 50) as i64 => {
                let entityPosition = convertToTilePos(x + self.screenPos.x, y + self.screenPos.y);
                let entityBuilder = EntityBuilder::new(self.currentEntityId, entityPosition);
                let entityRect = entityBuilder.getEntityRect();
                if self.globalEntities {
                    if let None = unsafe { deps.ctx.getEntityAtPositionGlobal(entityRect)} {
                        self.state.push(State::AttemptBuildEntity(entityBuilder));
                    }
                } 
                else { 
                    if let None = unsafe { deps.ctx.getEntityAtPositionActiveScreen(entityRect)} {
                        self.state.push(State::AttemptBuildEntity(entityBuilder));
                    }
                }
            },
            Event::MouseButtonDown {mouse_btn: MouseButton::Right, x, y, ..}
            if (y as i64) < (self.screenRect.height() - 50) as i64 => {
                let (x, y) = ((x + self.screenPos.x) / 50 * 50, (y + self.screenPos.y) / 50 * 50);
                let clickRect = Rect::new(x, y, 50, 50);
                unsafe {
                    let entity = if self.globalEntities { deps.ctx.getEntityAtPositionGlobal(clickRect)}
                    else {deps.ctx.getEntityAtPositionActiveScreen(clickRect)};
                    if let Some(id) = entity {
                        deps.ctx.removeEntity(id);
                    }
                }
            },
            Event::KeyDown{scancode: Some(Scancode::Left), ..} => {
                self.incEntity(-1);
            },
            Event::KeyDown{scancode: Some(Scancode::Right), ..} => {
                self.incEntity(1);
            },
            Event::KeyDown{scancode: Some(Scancode::E), ..} => {
                self.state.pop();
            },
            Event::KeyDown{scancode: Some(Scancode::G), ..} => {
                self.globalEntities = !self.globalEntities;
            },
            
            _ => self.matchCommon(event, deps),

        }
    }
    fn doRestrictedEntityPlacementEvents(&mut self, event: Event, deps: &mut EditorContextDeps) {
        match event {
            Event::KeyDown{scancode: Some(Scancode::E|Scancode::S|Scancode::A|Scancode::D|Scancode::M|Scancode::X), ..} => (),
			Event::KeyDown {scancode: Some(Scancode::Return), ..} => self.endList(deps),
            Event::MouseButtonDown {mouse_btn: MouseButton::Right, ..} => (),
            _ => self.doEntityPlacementEvents(event, deps),
        }
    }
	fn build<'a>(&mut self, signal: TileBuilderSignals, deps: &mut EditorContextDeps) {
		match signal {
			TileBuilderSignals::GetCoordinate(tmpMessage) => {
				self.state.push(State::GetCoordinate);
				*deps.fontTexture = Some(createText(&tmpMessage, deps.textureCreator, deps.font));
			}
			TileBuilderSignals::GetUserUsize(tmpMessage) => {
				self.state.push(State::GetUserUsize);
				self.textInput.start();
				self.message = String::from(tmpMessage);
				self.messageLen = tmpMessage.len() - 1;
				*deps.fontTexture = Some(createText(&self.message, deps.textureCreator, deps.font));
			},
			TileBuilderSignals::Complete(tile, pos) => {
				self.currentTile = tile;
				self.state.pop();
                if let State::GetTile = self.state.last().unwrap() {
                    self.state.pop();
                    match self.state.last_mut().unwrap() {
                        State::AttemptBuildEntity(ref mut builder) => builder.addTile(self.currentTile.clone(), pos),
                        State::Idle => deps.ctx.getMapMut().changeTile(pos, self.currentTile.clone()),
                        _ => unimplemented!(),
                    };
                }
                else {unreachable!()}
			},
			TileBuilderSignals::InvalidId => (),
		}	
	}
	fn buildEntity<'a>(&mut self, signal: EntityBuilderSignals<'a>, deps: &mut EditorContextDeps<'a, '_, '_, '_, '_, '_, '_, '_, '_,>) {
		match signal {
			EntityBuilderSignals::Complete(Ok(entity)) if self.globalEntities => {
				if let Some(State::AttemptBuildEntity(builder)) = self.state.pop() {
                    match self.state.pop().unwrap() {
    					State::EntityPlacement => builder.addEntityGlobal(deps.ctx, entity),
                        State::MakeEntityInactive => {
							let id = builder.addEntityInactive(deps.ctx, entity).unwrap();
                            if let Some(State::AttemptBuildEntity(mut builder)) = self.state.pop() {
                                builder.addInactiveEntity(id, self.globalEntities);
                            }
                            else {unreachable!()}
                        },
                        _ => unreachable!(),
                    }
                }
			},
			EntityBuilderSignals::Complete(Ok(entity)) => {
				if let Some(State::AttemptBuildEntity(builder)) = self.state.pop() {
                    match self.state.pop().unwrap() {
    					State::EntityPlacement => builder.addEntityActiveScreen(deps.ctx, entity),
                        State::MakeEntityInactive => {
							let id = builder.addEntityInactive(deps.ctx, entity).unwrap();
                            if let Some(State::AttemptBuildEntity(ref mut builder)) = self.state.last_mut() {
                                builder.addInactiveEntity(id, self.globalEntities);
                            }
                            else {unreachable!()}
                        },
                        _ => unreachable!(),
                    }
                }
            },
			EntityBuilderSignals::Complete(Err(e)) => eprintln!("Entity could not be placed because of error (details below)\n{}", e),
            EntityBuilderSignals::GetTile(msg) => {
                *deps.fontTexture = Some(createText(msg, deps.textureCreator, deps.font));
                self.state.push(State::GetTile);
            },
			EntityBuilderSignals::GetEntity(msg) => {
				*deps.fontTexture = Some(createText(msg, deps.textureCreator, deps.font));
				self.state.push(State::GetEntityID);
			}
            EntityBuilderSignals::MakeEntityInactive(msg) => {
                *deps.fontTexture = Some(createText(msg, deps.textureCreator, deps.font));
                self.state.push(State::MakeEntityInactive);
            }
			EntityBuilderSignals::InvalidId => eprintln!("Entity could not be placed because of invalid entity id produced by editor"),
		}
	}
	fn matchCommon(&mut self, event: Event, deps: &mut EditorContextDeps) {
		match event {
            Event::KeyDown{scancode: Some(Scancode::S), ..} => {
                let mut ser = Serializer::new(File::create(deps.filename).unwrap());
                unsafe {InnerGameContext::fromGameContext(deps.ctx)}.serialize(&mut ser).unwrap();
                let mut f = ser.into_inner();
                f.flush().unwrap();
            },	
            Event::KeyDown{scancode: Some(Scancode::A), ..} => {
                deps.ctx.getMapMut().decrementCurrentScreen();
                *deps.idTexture = Some(createText(&deps.ctx.getMap().getActiveScreenId().to_string(), deps.textureCreator, deps.font));
            },
            Event::KeyDown{scancode: Some(Scancode::D), ..} => {
                deps.ctx.getMapMut().incrementCurrentScreen();
                *deps.idTexture = Some(createText(&deps.ctx.getMap().getActiveScreenId().to_string(), deps.textureCreator, deps.font));
            },
            Event::KeyDown{scancode: Some(Scancode::Escape), ..} => {
				if self.state.len() > 1 {
	                self.state.pop();
				}
                *deps.idTexture = None;
            },
            Event::KeyDown{scancode: Some(Scancode::M), ..} => {
                self.state.push(State::MoveScreen);
                *deps.idTexture = None;
            },
            Event::KeyDown{scancode: Some(Scancode::X), ..} => {
                self.state.push(State::UserConfirmDelete);
                self.textInput.start();
                self.message = String::from("Are you sure you want to delete this map? (y/n): ");
                self.messageLen = self.message.len();
                *deps.fontTexture = Some(createText(&self.message, deps.textureCreator, deps.font));
            },

            Event::KeyDown {scancode: Some(Scancode::H), ..} => 
                self.screenPos.offset(self.screenRect.center().x.clamp(0, self.screenPos.x) * -1, 0),
            
            Event::KeyDown {scancode: Some(Scancode::J), ..} =>
                self.screenPos.offset(0, self.screenRect.center().y.clamp(0, self.screenPos.y) * -1),
        
            Event::KeyDown {scancode: Some(Scancode::K), ..} =>
                self.screenPos.offset(0, self.screenRect.center().y.min(deps.ctx.getMap().getMaxScreenCoords().1 as i32 - self.screenPos.bottom()).max(0)),
            
            Event::KeyDown {scancode: Some(Scancode::L), ..} => 
                self.screenPos.offset(self.screenRect.center().x.min(deps.ctx.getMap().getMaxScreenCoords().0 as i32 - self.screenPos.right()).max(0), 0),
			_ => (),
		};
	}
    fn incTile(&mut self, amt: i32) {
        if self.currentTileId as i32 + amt <= MAX_TILE_IDX as i32 && self.currentTileId as i32 + amt >= 0 {
            self.currentTileId = (self.currentTileId as i32 + amt) as u16;
            self.previewTile = Tile::preview(self.currentTileId);
        }
    }
    fn incCollision(&mut self, amt: isize) {
        if self.currentCollision as isize + amt <= MAX_COLLISION_IDX as isize && self.currentCollision as isize + amt >= 0 {
            self.currentCollision = (self.currentCollision as isize + amt) as usize;
        }
    }
    fn incEntity(&mut self, amt: i32) {
        if self.currentEntityId as i32 + amt <= MAX_ENTITY_IDX as i32 && self.currentEntityId as i32 + amt >= 0 {
            self.currentEntityId = (self.currentEntityId as i32 + amt) as u16;
        }
    }
	fn endList(&mut self, deps: &mut EditorContextDeps) {
		*deps.fontTexture = None;
		self.state.pop();
		if let Some(State::AttemptBuildEntity(ref mut builder)) = self.state.last_mut() {
			builder.endList();
		}
		else if let Some(State::AttemptBuild(ref mut builder)) = self.state.last_mut() {
			unimplemented!();
		}
		else {
			unreachable!();
		}
	}
}

fn convertToTilePos(x: i32, y: i32) -> (u16, u16) {
    ((x / 50) as u16, (y / 50) as u16)

}


pub fn createText<'a>(message: &str, textureCreator: &'a TextureCreator<WindowContext>, font: &Font) -> Texture<'a> {
	font.render(message).shaded(Color::BLACK, Color::WHITE).unwrap().as_texture(textureCreator).unwrap()
}

enum State {
	GetUserUsize,
	GetCoordinate,
    GetTile,
	GetEntityID,
    MakeEntityInactive,
	UserConfirmDelete,
	ViewMap,
	NewScreen,
	MoveScreen,
	AttemptBuild(TileBuilder),
	AttemptBuildEntity(EntityBuilder),
	Idle,
	EntityPlacement,
}

