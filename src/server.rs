//! `ChatServer` is an actor. It maintains list of connection client session.
//!  Peers send messages to other peers through `ChatServer`.

use actix::prelude::*;
use na::Vector2;
use rand::{self, Rng, ThreadRng};
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

/// Message for chat server communications
#[derive(Message)]
pub struct Message(pub String);

/// New chat session is created
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

#[derive(Message)]
pub struct KeysMessage {
    pub id: usize,
    pub keys: Vector2<f32>,
}

pub struct Player {
    pub key: Vector2<f32>,
    pub pos: Vector2<f32>,
}

pub trait Mob {
    fn update(&mut self);
}
pub enum Mobs {
    Skeleton { pos: Vector2<f32> },
}
impl Mob for Mobs {
    fn update(&mut self) {
        match self {
            Mobs::Skeleton { pos } => {
                let mut rng = rand::thread_rng();

                pos.x += rng.gen::<f32>() * 10.0 - 5.0;
                pos.y += rng.gen::<f32>() * 10.0 - 5.0;

                pos.x = pos.x.max(0.0).min(800.0);
                pos.y = pos.y.max(0.0).min(800.0);
            }
        }
    }
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
/// implementation is super primitive
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    players: HashMap<usize, Player>,
    mobs: Vec<Mobs>,
    rng: RefCell<ThreadRng>,
}

impl Default for ChatServer {
    fn default() -> ChatServer {
        let mut rng = rand::thread_rng();

        ChatServer {
            sessions: HashMap::new(),
            players: HashMap::new(),
            mobs: (0..rng.gen_range(10, 100))
                .map(|_| Mobs::Skeleton {
                    pos: Vector2::new(rng.gen::<f32>() * 800.0, rng.gen::<f32>() * 800.0),
                })
                .collect(),
            rng: RefCell::new(rng),
        }
    }
}
#[derive(Serialize)]
struct ClientPlayer {
    id: usize,
    pos: Vector2<f32>,
}
#[derive(Serialize)]
struct ClientMob {
    pos: Vector2<f32>,
    t: String,
}
#[derive(Serialize)]
struct Playfield {
    players: Vec<ClientPlayer>,
    mobs: Vec<ClientMob>,
}
impl ChatServer {
    /// Send message to all users
    fn send_message(&self, message: &str) {
        for (_, addr) in &self.sessions {
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
            for m in &mut act.mobs {
                m.update();
            }
            let playfield = Playfield {
                players: act.players
                    .iter()
                    .map(|(i, p)| ClientPlayer { id: *i, pos: p.pos })
                    .collect(),
                mobs: act.mobs
                    .iter()
                    .map(|m| match m {
                        Mobs::Skeleton { pos } => ClientMob {
                            t: "skeleton".to_string(),
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
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
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
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // register session with random id
        let id = self.rng.borrow_mut().gen::<usize>();
        self.sessions.insert(id, msg.addr);
        self.players.insert(
            id,
            Player {
                key: Vector2::new(0.0, 0.0),
                pos: Vector2::new(300.0, 200.0),
            },
        );
        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // remove address
        self.sessions.remove(&msg.id);
        self.players.remove(&msg.id);
        self.send_message(&::serde_json::to_string(&msg).unwrap())
    }
}

/// Handler for Message message.
impl Handler<Message> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Message, _: &mut Context<Self>) {
        self.send_message(&msg.0);
    }
}
impl Handler<KeysMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: KeysMessage, _: &mut Context<Self>) {
        if let Some(p) = self.players.get_mut(&msg.id) {
            p.key = msg.keys
        }
    }
}
