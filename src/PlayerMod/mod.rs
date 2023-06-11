extern crate sdl2;

use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::{Rect, Point};

use serde::{Serialize, Deserialize};

use std::io;

mod SignalsMod;

pub use SignalsMod::{SignalsBuilder, Signals, Mapping};

use crate::SpriteLoader::{Animations, Sprites};
use crate::{Direction, Map, CollisionType, Vector, GameContext, ID};
use crate::Entities::Traits::{Collision, EntityTraitsWrappable, Entity, Counter, RegisterID};
use crate::Entities::{BoxCode, RefCode, RefCodeMut, TypedID, Rock};
use crate::EventProcessor::{CollisionMsg, CounterMsg, Envelope, PO, Key};
use crate::MapMod::{self, Tile};

const SWORD_FRAMES: &'static[&'static str] = &[
	"Resources/Images/Sword__half.png",
];

const HEALTH_FRAMES: &'static[&'static str] = &[
	"Resources/Images/Health_full.png",
	"Resources/Images/Health_half.png",
	"Resources/Images/Health_empty.png",
];

enum HEALTH_IDX {
	Full = 0,
	Half,
	Empty,
}

const SWORD_DOWN: (i32, i32, u32, u32) = (10, 43, 30, 30);
const SWORD_RIGHT: (i32, i32, u32, u32) = (30, 5, 30, 30);
const SWORD_LEFT: (i32, i32, u32, u32) = (-10, 5, 30, 30);
const SWORD_UP: (i32, i32, u32, u32) = (0, -10, 50, 50);

const SWORD_DOWN_COLLISION: (i32, i32, u32, u32) = (23, 43, 4, 16);
const SWORD_RIGHT_COLLISION: (i32, i32, u32, u32) = (49, 5, 11, 16);
const SWORD_LEFT_COLLISION: (i32, i32, u32, u32) = (-10, 5, 11, 16);
const SWORD_UP_COLLISION: (i32, i32, u32, u32) = (27, -10, 6, 27);

const NAMES: &'static[&'static str] = &[
	"Ninja float",
	"Ninja right float",
	"Ninja left float",
	"Ninja up float",
	"Ninja attack",
	"Ninja right attack",
	"Ninja left attack",
	"Ninja up attack",
	"Ninja sink",
	"Ninja burn",
];

fn relTupleToRect(tuple: (i32, i32, u32, u32), position: (i32, i32)) -> Rect {
	Rect::new(
		tuple.0 + position.0,
		tuple.1 + position.1,
		tuple.2,
		tuple.3
	)
}

enum ANIMATION_IDX {
	DownFloat = 0,
	RightFloat,
	LeftFloat,
	UpFloat,
	DownAttack,
	RightAttack,
	LeftAttack,
	UpAttack,
	NinjaSink,
	NinjaBurn,
}

#[derive(Serialize, Deserialize)]
pub struct InnerPlayer {
	id: ID,
	direction: Direction,
	timer: u32,
	idle: bool,
	velocity: Vector,
    position: Vector,
	hitbox: (i32, i32, u32, u32),
	renderPosition: (i32, i32, u32, u32),
	attackTimer: u32,
	attacking: bool,
	health: i32,
	iframes: u32,
}

impl InnerPlayer {
	pub fn fromPlayer(player: &Player) -> InnerPlayer {
		let &Player {id, direction, timer, idle, velocity, position, hitbox, renderPosition, attackTimer, attacking, health, iframes, ..} = player;
		InnerPlayer {id:id.getID(), direction, timer, idle, velocity, position, hitbox: hitbox.into(), renderPosition: renderPosition.into(), attackTimer, attacking, health, iframes}
	}
}

#[derive(Debug)]
pub struct Player<'a> {
	id: TypedID<'a, Self>,
	animations: Animations<'a>,
	direction: Direction,
	timer: u32,
	idle: bool,
	velocity: Vector,
    position: Vector,
	hitbox: Rect,
	renderPosition: Rect,
	attackTimer: u32,
	attacking: bool,
	health: i32,
	iframes: u32,
	sword: Sprites<'a>,
	healthSprites: Sprites<'a>,
	hitSwitchLastFrame: bool,
	keys: u8,
	abyss: u16,
	burn: u16,
	respawn: Vector,
	elevated: u8,
	maybeBurn: bool,
	maybeAbyss: bool,
}
#[derive(Debug)]
pub struct PlayerData {
	//transition: Option<Rect>,
	nextPos: Vector,
	stopHitSwitch: bool,
	dmg: i32,
	keys: u8,
	burn: bool,
	abyss: bool,
}

impl PlayerData {
	fn doCollision(&mut self, player: &Player, map: &Map, po: &PO) {
		let mut tmp = player.hitbox;
		tmp.reposition(self.nextPos + Vector(2f32, 2f32));
		let mut iter = map.calculateCollisionBounds(tmp);

		while let Some((location, tile)) = map.collide(&mut iter) {
			match tile.getCollisionType() {
				CollisionType::KeyBlock if self.keys > 0 => {
					po.spawnTile(Tile::default(), location);
					self.keys -= 1;
				}
				CollisionType::Block | CollisionType::SwitchToggleGate(..) | CollisionType::SwitchTriggerGen(..) | CollisionType::KeyBlock => {
					let eject = MapMod::blockCollide(location, tmp, map);
					self.nextPos += eject;
					tmp.reposition(self.nextPos + Vector(2f32, 2f32));
				},
				CollisionType::Hit(dmg) => {
					self.dmg = dmg;
				}
				CollisionType::Abyss if player.elevated == 0 => {
					let (x, y) = tmp.center().into();
					if ((x / 50) as u16, (y / 50) as u16) == location {
						self.abyss = true;
						tmp.reposition((location.0 as i32 * 50 + 2, location.1 as i32 * 50 + 2));
						self.nextPos = Vector::from(<Point as Into<(i32, i32)>>::into(tmp.top_left())) - Vector(2f32, 2f32);
					}
				}
				CollisionType::Burn if player.elevated == 0 => {
					if Rect::new(location.0 as i32 * 50, location.1 as i32 * 50, 50, 50).contains_point(tmp.center()) {
						self.burn = true;
						tmp.reposition((location.0 as i32 * 50 + 2, location.1 as i32 * 50 + 2));
						self.nextPos = Vector::from(<Point as Into<(i32, i32)>>::into(tmp.top_left())) - Vector(2f32, 2f32);
					}
				}
				CollisionType::Key => {
					po.spawnTile(Tile::default(), location);
					self.keys += 1;
				}
				CollisionType::SpawnGate(location) => po.spawnTiles(Tile::gate(), (location.0, location.1), (location.2, location.3)),
				CollisionType::ClearTiles(location) => po.spawnTiles(Tile::default(), (location.0, location.1), (location.2, location.3)),
				_ => (),
			}
		}
		if player.attacking || player.attackTimer > 0 {
			let tmp = relTupleToRect(player.getSwordCollision(), (self.nextPos + Vector(2f32, 2f32)).into());
			let mut iter = map.calculateCollisionBounds(tmp);

			while let Some((location, tile)) = map.collide(&mut iter) {
				match tile.getCollisionType() {
					CollisionType::SwitchToggleGate(..) if player.hitSwitchLastFrame => self.stopHitSwitch = false,
					CollisionType::SwitchTriggerGen(..) if player.hitSwitchLastFrame => self.stopHitSwitch = false,
					CollisionType::SwitchToggleGate(range) => {
						let spawnedTile = if let CollisionType::Block = map.getScreen(map.getActiveScreenId()).unwrap().getTile((range.0, range.1)).getCollisionType() {Tile::default()}
						else {Tile::gate()};
						po.spawnTiles(spawnedTile, (range.0, range.1), (range.2, range.3));
						let id = match tile.getId() {
							3 => 4,
							4 => 3,
							_ => tile.getId(),
						};
						po.spawnTile(Tile::new(id, tile.getCollisionType()), location);
						self.stopHitSwitch = false;
					},
					CollisionType::SwitchTriggerGen(id) => {
						po.sendCounterMsg(Envelope::new(CounterMsg(i32::MIN), id, player.id.getID()));
						let id = match tile.getId() {
							3 => 4,
							4 => 3,
							_ => tile.getId(),
						};
						po.spawnTile(Tile::new(id, tile.getCollisionType()), location);
						self.stopHitSwitch = false;
					}
					_ => (),
				}
			}
		}
	}
	fn doEntityCollision(&mut self, player: &Player, po: &PO, mut key: Key) -> Key {
		//Sword
		for id in po.getCtx().getCollisionList(player.id.getID().sub(1)).filter(|id| id.mask() != player.id.getID().mask()) {
			po.sendCollisionMsg(Envelope::new(CollisionMsg::Damage(5), id, player.id.getID().sub(1)));
		}
		for id in po.getCtx().getCollisionList(player.id.getID()).filter(|id| id.mask() != player.id.getID().mask()) {
			let res = po.getEntity(id.mask(), key);
			key = res.1;
			let entity = if let Some(entity) = res.0 {entity} else {panic!("{:?}", id)};
			let res = entity.collideWith(player.id.getID(), po, key);
			key = res.1;
			if let Some(msg) = res.0 {
				if let None = po.getCtx().getHolder().getTyped(TypedID::<Rock>::new(msg.getSender())) {
					po.sendCollisionMsg(msg);
				}
			}
		}
		key
	}
}

impl<'a> Player<'a> {
    pub fn new(creator: &'a TextureCreator<WindowContext>, positionX: f32, positionY: f32) -> io::Result<BoxCode<'a>> {
        let (direction, velocity, position, timer, idle, attackTimer, attacking, health, iframes, hitSwitchLastFrame, keys, abyss, respawn, burn, elevated, maybeBurn, maybeAbyss) = (
            Direction::Down, 
            Vector(0f32, 0f32), 
            Vector(positionX, positionY),
			0u32,
			true,
			0u32,
			false,
			50,
			0,
			false,
			0,
			0,
			Vector(0f32, 0f32),
			0,
			0,
			false, 
			false,
        );
		let animations = Animations::new("Resources/Images/Ninja.anim", NAMES, creator)?;
		let sword = Sprites::new(creator, SWORD_FRAMES)?;
		let healthSprites = Sprites::new(creator, HEALTH_FRAMES)?;
		let renderPosition = Rect::new(positionX.round() as i32, positionY.round() as i32, 50, 50);
		let hitbox = Rect::new(positionX.round() as i32 + 2, positionY as i32 + 2, 46, 46);

        Ok(
			BoxCode::Player(
				Entity::new(
					Player {id: TypedID::new(ID::empty()), animations, direction, velocity, position, timer, idle, hitbox, renderPosition, attackTimer, sword, attacking, health, iframes, healthSprites, hitSwitchLastFrame, keys, abyss, respawn, burn, elevated, maybeAbyss, maybeBurn},
					PlayerData {
						keys,
						nextPos: position,
						stopHitSwitch: true,
						dmg: 0,
						abyss: false,
						burn: false,
					},
				)
			)
		)
    }
	pub fn fromInner(inner: InnerPlayer, creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		Ok(BoxCode::Player(
			Entity::new(
				Player {
					id: TypedID::new(inner.id),
					animations: Animations::new("Resources/Images/Ninja.anim", NAMES, creator)?,
					direction: inner.direction,
					velocity: inner.velocity,
					position: inner.position,
					timer: inner.timer,
					idle: inner.idle,
					hitbox: Rect::from(inner.hitbox),
					attackTimer: inner.attackTimer,
					health: inner.health,
					renderPosition: Rect::from(inner.renderPosition),
					iframes: inner.iframes,
					sword: Sprites::new(creator, SWORD_FRAMES)?,
					healthSprites: Sprites::new(creator, HEALTH_FRAMES)?,
					attacking: inner.attacking,
					hitSwitchLastFrame: false,
					keys: 0,
					abyss: 0,
					respawn: Vector(0f32, 0f32),
					burn: 0,
					elevated: 0,
					maybeAbyss: false,
					maybeBurn: false,
				},
				PlayerData {
					keys: 0,
					nextPos: inner.position,
					stopHitSwitch: true,
					dmg: 0,
					abyss: false,
					burn: false,
				}
			)
		))
	}
	pub fn collidesStatic(&self, hitbox: Rect) -> bool {
		self.hitbox.has_intersection(hitbox)
	}

	fn getSwordCollision(&self) -> (i32, i32, u32, u32) {
		match self.direction {
			Direction::Up => SWORD_UP_COLLISION,
			Direction::Down => SWORD_DOWN_COLLISION,
			Direction::Left => SWORD_LEFT_COLLISION,
			Direction::Right => SWORD_RIGHT_COLLISION
		}
	}

	pub fn updatePositionsPO(&mut self, po: &mut PO) {
		if self.abyss == 0 && self.burn <= 360 {self.respawn = self.position;}
		self.renderPosition.reposition(self.position);
		let prevHitbox = self.hitbox;
		self.hitbox.reposition(self.position + Vector(2f32, 2f32));
		po.updatePosition(self.id.getID(), self.hitbox, prevHitbox);
		if self.attacking || self.attackTimer > 0 {
			let swordBox = self.getSwordCollision();
			po.updatePosition(self.id.getID().sub(1), relTupleToRect(swordBox, self.hitbox.top_left().into()), relTupleToRect(swordBox, prevHitbox.top_left().into()));
		}
	}

	pub fn updatePositionsCtx(&mut self, ctx: &mut GameContext) {
		if self.abyss == 0 && self.burn <= 360 {self.respawn = self.position;}
		self.renderPosition.reposition(self.position);
		let prevHitbox = self.hitbox;
		self.hitbox.reposition(self.position + Vector(2f32, 2f32));
		ctx.updatePosition(self.id.getID(), self.hitbox, prevHitbox);
		if self.attacking || self.attackTimer > 0 {
			let swordBox = match self.direction {
				Direction::Up => SWORD_UP_COLLISION,
				Direction::Down => SWORD_DOWN_COLLISION,
				Direction::Left => SWORD_LEFT_COLLISION,
				Direction::Right => SWORD_RIGHT_COLLISION
			};
			ctx.updatePosition(self.id.getID().sub(1), relTupleToRect(swordBox, self.hitbox.top_left().into()), relTupleToRect(swordBox, prevHitbox.top_left().into()));
		}
	}

	pub fn transition(&mut self, ctx: &mut GameContext) -> bool {
		if let Some(hitbox) = ctx.getMapMut().transitionScreen(self.hitbox) {
			let point: (i32, i32) = hitbox.top_left().into();
			self.position = Vector::from(point);
			self.updatePositionsCtx(ctx);
			true
		}
		else {false}

	}

    pub fn signal(&mut self, signal: Signals) {
		match (signal.attack, self.attacking) {
			(Some(true), false) => {
				self.attackTimer = 21;
				self.attacking = true;
				self.idle = false;
			},
			(Some(false), true) => {
				self.attacking = false;
			},
			(Some(true), true) | (None, _) | (Some(false), false) => {},
		}
		match signal {
			Signals {up: Some(true), ..} => {
				self.direction = Direction::Up;
				self.velocity.1 = -3f32;
			},
			Signals {down: Some(true), ..} => {
				self.direction = Direction::Down;
				self.velocity.1 = 3f32;
			},
			Signals {up: Some(false), ..} => {
				self.velocity.1 = 0f32;
			},
			Signals {down: Some(false), ..} => {
				self.velocity.1 = 0f32;
			},
			_ => (),
		}
		match signal {
			Signals {left: Some(true), ..} => {
				self.direction = Direction::Left;
				self.velocity.0 = -3f32;
			},
			Signals {right: Some(true), ..} => {
				self.direction = Direction::Right;
				self.velocity.0 = 3f32;
			},
			Signals {left: Some(false), ..} => {
				self.velocity.0 = 0f32;
			},
			Signals {right: Some(false), ..} => {
				self.velocity.0 = 0f32;
			},
			_ => (),
		}
	}
	pub fn getPosition(&self) -> Vector {
		self.position
	}
}

impl<'a> Collision for Player<'a> {
	fn collide(&mut self, msg: Envelope<CollisionMsg>, po: &PO) {
		match msg.getMsg() {
			CollisionMsg::Damage(dmg) => {
				if self.iframes == 0 {
					self.health -= dmg;
					self.iframes = 90;
				}
			},
			CollisionMsg::Ground(hitbox, dp) => {
				if hitbox.contains_point(self.hitbox.center()) {

					self.elevated = 2;
					self.position += *dp;
				}
			}
		};
	}
	fn collideWith(&self, other: ID, po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key) {
		(None, key)
	}
}

impl<'a> Counter for Player<'a> {}
impl<'a> RegisterID for Player<'a> {}

impl<'a> EntityTraitsWrappable<'a> for Player<'a> {
	type Data = PlayerData;
	fn setID(&mut self, id: TypedID<'a, Self>) {
		self.id = id;
	}
	fn mapCodeMut<'b>(code: RefCodeMut<'a, 'b>) -> Option<&'b mut Self> {
		if let RefCodeMut::Player(p) = code {Some(p as &mut Self)}
		else {None}
	}
	fn mapCode<'b>(code: RefCode<'a, 'b>) -> Option<&'b Self> {
		if let RefCode::Player(p) = code {Some(p as &Self)}
		else {None}
	}
	fn getData(&self, data: &mut Self::Data, po: &PO, key: Key) -> Key {
		//data.transition = ctx.getMap().transitionScreen(self.hitbox);
		data.abyss = false;
		data.burn = false;
		if self.abyss > 0 || self.burn > 360 {
			data.nextPos = Vector(0f32, 0f32);
			return key;
		}
		data.keys = self.keys;
		data.stopHitSwitch = true;
		data.dmg = 0;
		data.nextPos = if self.idle {
			self.position + self.velocity
		} else {self.position};
		let origPosition = self.position;
		data.doCollision(self, po.getCtx().getMap(), po);
		let key = data.doEntityCollision(self, po, key);
		data.nextPos -= origPosition;
		key
	}
	fn update(&mut self, data: &Self::Data, po: &mut PO) {
		if data.abyss && self.maybeAbyss {
			self.abyss = 31;
			self.health -= 5;
			self.maybeAbyss = false;
		}
		if data.burn && self.maybeBurn {
			self.burn = 391;
			self.health -= 5;
			self.animations.changeAnimation(ANIMATION_IDX::NinjaSink as usize);
			self.maybeBurn = false;
		}
		self.position += data.nextPos;
		self.updatePositionsPO(po);
		if self.abyss > 0 {
			self.abyss -= 1;
			if self.abyss == 0 {
				self.position = self.respawn;
				self.updatePositionsPO(po);
			}
			return;
		}
		if self.burn > 0 {
			self.burn -= 1;
			if self.burn == 360 {
				self.position = self.respawn;
				self.updatePositionsPO(po);
				self.animations.changeAnimation(ANIMATION_IDX::NinjaBurn as usize);
			}
			if self.burn < 360 && self.burn % 10 == 0 {
				self.animations.update();
			}
		}
		if self.burn > 360 {
			if self.burn > 363 && self.burn % 7 == 6 {
				self.animations.update();
			}
			return;
		}

		if self.burn % 120 == 1 {
			self.health -= 5;
		}
		self.maybeAbyss = data.abyss;
		self.maybeBurn = data.burn;

		if self.elevated > 0 {self.elevated -= 1;}
	
		self.hitSwitchLastFrame = !data.stopHitSwitch;

		if self.iframes == 0 {
			self.health += data.dmg;
			if data.dmg < 0 {
				self.iframes = 90;
			}
		}

		self.keys = data.keys;

		if self.idle && self.burn == 0 {match self.direction {
			Direction::Up => {self.animations.changeAnimation(ANIMATION_IDX::UpFloat as usize);},
			Direction::Down => {self.animations.changeAnimation(ANIMATION_IDX::DownFloat as usize);},
			Direction::Left => {self.animations.changeAnimation(ANIMATION_IDX::LeftFloat as usize);},
			Direction::Right => {self.animations.changeAnimation(ANIMATION_IDX::RightFloat as usize);},
		}}

		if self.burn == 0 {self.timer += 1;}
		if self.timer > 20 {
			self.timer = 0;
			self.animations.update();
		}
		if self.attackTimer > 0 {
			self.attackTimer -= 1;
		}	
		if self.burn == 0 && (self.attackTimer > 0 || self.attacking) {
			match self.direction {
				Direction::Up => {self.animations.changeAnimation(ANIMATION_IDX::UpAttack as usize);},
				Direction::Down => {self.animations.changeAnimation(ANIMATION_IDX::DownAttack as usize);},
				Direction::Left => {self.animations.changeAnimation(ANIMATION_IDX::LeftAttack as usize);},
				Direction::Right => {self.animations.changeAnimation(ANIMATION_IDX::RightAttack as usize);},
			};
			// Add attack code here
		}
		if !self.idle && !self.attacking && self.attackTimer == 0 {
			po.removeCollision(self.id.getID().sub(1), relTupleToRect(self.getSwordCollision(), self.hitbox.top_left().into()));
			self.idle = true;
		}
		if self.iframes > 0 {self.iframes -= 1;}
	}
	fn needsExecution(&self) -> bool {true}
	fn tick(&mut self) {}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		let mut health = self.health;
		let mut healthRect = Rect::new(15, 15, 15, 15);
		for _ in 0..5 {
			if health >= 10 {
				health -= 10;
				self.healthSprites.getSprite(HEALTH_IDX::Full as usize)
			}
			else if health > 0 {
				health -= 10;
				self.healthSprites.getSprite(HEALTH_IDX::Half as usize)
			}
			else {
				self.healthSprites.getSprite(HEALTH_IDX::Empty as usize)
			}.draw(canvas, healthRect, false, false);
			healthRect.reposition((healthRect.x() + 15, healthRect.y()));
		}
		if (self.iframes / 10) % 2 == 1 {return;}
		if self.abyss > 0 {
			let mut tmp = self.renderPosition;
			tmp.resize(50 * self.abyss as u32 / 30, 50 * self.abyss as u32 / 30);
			tmp.center_on(self.renderPosition.center());
			self.animations.drawNextFrame(canvas, tmp);
			return;
		}
		if self.burn > 360 {
			if self.burn > 370 {
				self.animations.drawNextFrame(canvas, self.renderPosition);
			}
			return;
		}
		if self.attacking || self.attackTimer > 0 {match self.direction {
			Direction::Up => {
				self.sword.getSprite(0).draw(canvas, Rect::new (
					SWORD_UP.0 + self.renderPosition.x(),
					SWORD_UP.1 + self.renderPosition.y(),
					SWORD_UP.2,
					SWORD_UP.3
				), false, false);
				self.animations.drawNextFrame(canvas, self.renderPosition);
			},
			Direction::Down => {
				self.animations.drawNextFrame(canvas, self.renderPosition);
				self.sword.getSprite(0).draw(canvas, Rect::new(
					SWORD_DOWN.0 + self.renderPosition.x(),
					SWORD_DOWN.1 + self.renderPosition.y(),
					SWORD_DOWN.2,
					SWORD_DOWN.3
				), false, true);
			},
			Direction::Left => {
				self.sword.getSprite(0).draw(canvas, Rect::new (
					SWORD_LEFT.0 + self.renderPosition.x(),
					SWORD_LEFT.1 + self.renderPosition.y(),
					SWORD_LEFT.2,
					SWORD_LEFT.3
				), false, false);
				self.animations.drawNextFrame(canvas, self.renderPosition);
			},
			Direction::Right => {
				self.sword.getSprite(0).draw(canvas, Rect::new (
					SWORD_RIGHT.0 + self.renderPosition.x(),
					SWORD_RIGHT.1 + self.renderPosition.y(),
					SWORD_RIGHT.2,
					SWORD_RIGHT.3
				), false, false);
				self.animations.drawNextFrame(canvas, self.renderPosition);
					
			}
		}}
		else {
			self.animations.drawNextFrame(canvas, self.renderPosition);
		}
	}

}

