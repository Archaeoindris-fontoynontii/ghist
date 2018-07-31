//! `GameServer` is an actor. It maintains list of connection client session.
//!  Peers send messages to other peers through `GameServer`.

use actix::prelude::*;
use na::Vector2;
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use ncollide2d::world::{
    CollisionGroups, CollisionObjectHandle, CollisionWorld, GeometricQueryType,
};
use rand::{self, Rng, ThreadRng};
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

/// Message for game server communications
#[derive(Message)]
pub struct Message(pub String);

/// New game session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message, Serialize)]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Deserialize)]
pub enum ClientMessage {
    Name(String),
    Keys(Vector2<f32>),
    Click(bool),
}

#[derive(Message)]
pub struct ServerMessage {
    pub id: usize,
    pub m: ClientMessage,
}

pub struct Player {
    pub key: Vector2<f32>,
    pub pos: Vector2<f32>,
    pub health: u8,
    pub mouse: bool,
}

pub trait Mob {
    fn update(&mut self);
}
pub enum Mobs {
    Skeleton { pos: Vector2<f32>, health: u8 },
}
impl Mob for Mobs {
    fn update(&mut self) {
        match self {
            Mobs::Skeleton { pos, .. } => {
                let mut rng = rand::thread_rng();

                pos.x += rng.gen::<f32>() * 10.0 - 5.0;
                pos.y += rng.gen::<f32>() * 10.0 - 5.0;

                pos.x = pos.x.max(0.0).min(800.0);
                pos.y = pos.y.max(0.0).min(800.0);
            }
        }
    }
}

/// `GameServer` responsible for coordinating game sessions.
/// implementation is super primitive
pub struct GameServer {
    sessions: HashMap<usize, Recipient<Message>>,
    players: HashMap<usize, Player>,
    mobs: HashMap<usize, Mobs>,
    cw: CollisionWorld<f32, CollisionId>,      // Collision World
    cg: CollisionGroups,                       // Collision Groups
    ch: HashMap<usize, CollisionObjectHandle>, // Collision Handle
    rng: RefCell<ThreadRng>,
}

#[derive(Serialize)]
struct ClientPlayer {
    id: usize,
    pos: Vector2<f32>,
    health: u8,
}
#[derive(Serialize)]
struct ClientMob {
    id: usize,
    pos: Vector2<f32>,
    health: u8,
    t: String,
}
#[derive(Serialize)]
struct Playfield {
    players: Vec<ClientPlayer>,
    mobs: Vec<ClientMob>,
}
#[derive(Debug)]
enum CollisionId {
    Mob(usize),
    Player(usize),
}
impl GameServer {
    pub fn new() -> GameServer {
        let mut rng = rand::thread_rng();
        let mut cg = CollisionGroups::new();
        cg.set_membership(&[1]);
        GameServer {
            sessions: HashMap::new(),
            players: HashMap::new(),
            mobs: (0..rng.gen_range(5, 20))
                .map(|_| {
                    (
                        rng.gen::<usize>(),
                        Mobs::Skeleton {
                            pos: Vector2::new(rng.gen::<f32>() * 800.0, rng.gen::<f32>() * 800.0),
                            health: 128,
                        },
                    )
                })
                .collect(),
            cw: CollisionWorld::new(0.02),
            cg,
            ch: HashMap::new(),
            rng: RefCell::new(rng),
        }
    }
    /// Send message to all players
    fn send_message(&self, message: &str) {
        for addr in self.sessions.values() {
            let _ = addr.do_send(Message(message.to_owned()));
        }
    }
    fn tick(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::from_millis(5), |act, ctx| {
            for p in act.players.values_mut() {
                p.pos += p.key * 2.0;
                p.pos.x = p.pos.x.max(0.0).min(800.0);
                p.pos.y = p.pos.y.max(0.0).min(800.0);
            }
            for m in act.mobs.values_mut() {
                m.update();
            }

            act.collide_players();

            let playfield = Playfield {
                players: act
                    .players
                    .iter()
                    .map(|(i, p)| ClientPlayer {
                        id: *i,
                        pos: p.pos,
                        health: p.health,
                    })
                    .collect(),
                mobs: act
                    .mobs
                    .iter()
                    .map(|(i, m)| match m {
                        Mobs::Skeleton { pos, health } => ClientMob {
                            id: *i,
                            t: "skeleton".to_string(),
                            health: *health,
                            pos: *pos,
                        },
                    })
                    .collect(),
            };
            let serialized = ::serde_json::to_string(&playfield).unwrap();
            act.send_message(&serialized);

            act.tick(ctx);
        });
    }

    fn collide_players(&mut self) {
        for (id, player) in self.players.iter() {
            let position = ::na::Isometry2::new(player.pos, ::na::zero());
            if let Some(co) = self.ch.get_mut(id) {
                self.cw.set_position(*co, position)
            }
            // player size 112, 200
        }
        for event in self.cw.proximity_events() {
            let co1 = self.cw.collision_object(event.collider1).unwrap();
            let co2 = self.cw.collision_object(event.collider2).unwrap();
            if event.new_status == Proximity::Intersecting {
                println!("The players collides with another player.");
                if let (CollisionId::Player(i), CollisionId::Player(i2)) = (co1.data(), co2.data())
                {
                    if let Some(p) = self.players.get_mut(i) {
                        p.health = 64;
                    }
                    if let Some(p2) = self.players.get_mut(i2) {
                        p2.health = 64;
                    }
                }
            } else if event.new_status == Proximity::Disjoint {
                if let (CollisionId::Player(i), CollisionId::Player(i2)) = (co1.data(), co2.data())
                {
                    if let Some(p) = self.players.get_mut(i) {
                        p.health = 128;
                    }
                    if let Some(p2) = self.players.get_mut(i2) {
                        p2.health = 128;
                    }
                }
                println!("The player stops colliding with another player.");
            }
        }
        for _ in self.cw.contact_events() {
            panic!("This shouldn't happen");
        }

        self.cw.update()
        // skeleton size 84, 150
    }
}

/// Make actor from `GameServer`
impl Actor for GameServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.tick(ctx);
    }
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // register session with random id
        let id = self.rng.borrow_mut().gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // remove address
        self.sessions.remove(&msg.id);
        self.players.remove(&msg.id);
        self.send_message(&::serde_json::to_string(&msg).unwrap())
    }
}

impl Handler<ServerMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, _: &mut Context<Self>) {
        if let ClientMessage::Name(_) = msg.m {
            let p = Player {
                key: Vector2::new(0.0, 0.0),
                pos: Vector2::new(400.0, 400.0),
                health: 128,
                mouse: false,
            };
            let proximity_query = GeometricQueryType::Proximity(0.0);
            let position = ::na::Isometry2::new(p.pos, ::na::zero());
            // player size 112, 200
            let shape = ShapeHandle::new(Cuboid::new(Vector2::new(56.0, 100.0)));
            self.ch.insert(
                msg.id,
                self.cw.add(
                    position,
                    shape,
                    self.cg,
                    proximity_query,
                    CollisionId::Player(msg.id),
                ),
            );
            self.players.insert(msg.id, p);
        }
        if let Some(p) = self.players.get_mut(&msg.id) {
            match msg.m {
                ClientMessage::Keys(k) => p.key = k,
                ClientMessage::Click(b) => p.mouse = b,
                ClientMessage::Name(_) => (),
            }
        }
    }
}
