//! `ChatServer` is an actor. It maintains list of connection client session.
//!  Peers send messages to other peers through `ChatServer`.

use std::cell::RefCell;
use std::collections::HashMap;
use rand::{self, Rng, ThreadRng};
use actix::prelude::*;

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

/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
/// implementation is super primitive
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rng: RefCell<ThreadRng>,
}

impl Default for ChatServer {
    fn default() -> ChatServer {
        ChatServer {
            sessions: HashMap::new(),
            rng: RefCell::new(rand::thread_rng()),
        }
    }
}

impl ChatServer {
    /// Send message to all users
    fn send_message(&self, message: &str) {
        println!("Sent: {}", message);
        for (_, addr) in &self.sessions {
            let _ = addr.do_send(Message(message.to_owned()));
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // notify all users in same room
        self.send_message("Someone joined");

        // register session with random id
        let id = self.rng.borrow_mut().gen::<usize>();
        self.sessions.insert(id, msg.addr);

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
        // send message to other users
        self.send_message("Someone disconnected");
    }
}

/// Handler for Message message.
impl Handler<Message> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Message, _: &mut Context<Self>) {
        self.send_message(&msg.0);
    }
}

