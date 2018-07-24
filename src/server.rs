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
#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Deserialize)]
pub struct Keys {
    x: i8,
    y: i8,
}

#[derive(Message)]
pub struct KeysMessage {
    pub id: usize,
    pub keys: Keys,
}

pub struct Player {
    pub key: Keys, //Technically only needs 4 bits
    pub pos: Vector2<f32>,
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
/// implementation is super primitive
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    players: HashMap<usize, Player>,
    rng: RefCell<ThreadRng>,
}

impl Default for ChatServer {
    fn default() -> ChatServer {
        ChatServer {
            sessions: HashMap::new(),
            players: HashMap::new(),
            rng: RefCell::new(rand::thread_rng()),
        }
    }
}
#[derive(Serialize)]
struct ClientPlayer {
    id: usize,
    pos: Vector2<f32>,
}
#[derive(Serialize)]
struct Playfield {
    players: Vec<ClientPlayer>,
}
impl ChatServer {
    /// Send message to all users
    fn send_message(&self, message: &str) {
        for (_, addr) in &self.sessions {
            let _ = addr.do_send(Message(message.to_owned()));
        }
    }
    fn tick(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(0, 1), |act, ctx| {
            //act.send_message("\"left\"");
           // let players = ;
            for p in act.players.values_mut() {
                p.pos.x += p.key.x as f32;
                p.pos.y += p.key.y as f32;
            }
            let playfield = Playfield {
               players:act
                   .players
                   .iter()
                   .map(|(i, p)| ClientPlayer { id: *i, pos: p.pos })
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
        // for (i, p) in &self.players {
        //     msg.addr.do_send(Message(
        //         json!({
        //             "id": i,
        //             "pos": p.pos,
        //         }).to_string(),
        //     ));
        // }

        // register session with random id
        let id = self.rng.borrow_mut().gen::<usize>();
        self.sessions.insert(id, msg.addr);
        self.players.insert(id, Player {
            key: Keys { x: 0, y: 0 },
            pos: Vector2::new(300.0,200.0)
        });
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
        if let Some(p) = self.players.get_mut(&msg.id){
            p.key = msg.keys
        }
    }
}
