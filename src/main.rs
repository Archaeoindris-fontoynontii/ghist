extern crate rand;
extern crate nalgebra as na;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate actix;
extern crate actix_web;

use actix::*;
use actix_web::server::HttpServer;
use actix_web::*;

use std::time::Instant;

mod server;
use server::{Keys,KeysMessage};
/// This is our WebSocket route state, this state is shared with all route instances
/// via `HttpContext::state()`
struct WsChatSessionState {
    addr: Addr<server::ChatServer>,
}

/// Entry point for our route
fn chat_route(req: &HttpRequest<WsChatSessionState>) -> Result<HttpResponse> {
    ws::start(
        req,
        WsChatSession {
            id: 0,
            name: None,
        },
    )
}

struct WsChatSession {
    /// unique session id
    id: usize,
    /// peer name
    name: Option<String>,
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self, WsChatSessionState>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared across all
        // routes within application
        let addr: Addr<_> = ctx.address();
        ctx.state()
            .addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // notify chat server
        ctx.state().addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer WebSocket
impl Handler<server::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<ws::Message, ws::ProtocolError> for WsChatSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Pong(_) => println!("Ping"),
            ws::Message::Text(text) => {
                let m = text.trim();
                let k:Keys = serde_json::from_str(m).unwrap();

                let msg = if let Some(ref name) = self.name {
                    format!("{}: {}", name, m)
                } else {
                    m.to_owned()
                };
                // send message to chat server
                ctx.state().addr.do_send(server::KeysMessage{keys:k,id:self.id});
                ctx.state().addr.do_send(server::Message(msg))
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
        }
    }
}

fn main() {
    let sys = actix::System::new("ghist");

    // Start chat server actor in separate thread
    let server: Addr<_> = Arbiter::start(|_| server::ChatServer::default());

    // Create Http server with WebSocket support
    HttpServer::new(move || {
        // WebSocket sessions state
        let state = WsChatSessionState {
            addr: server.clone(),
        };

        App::with_state(state)
                .resource("/ws/", |r| r.f(chat_route))
                // static resources
                .handler("/", fs::StaticFiles::new("static/").unwrap().index_file("index.html"))
    }).bind("0.0.0.0:8080")
        .unwrap()
        .start();

    println!("Started http server: http://localhost:8080");
    let _ = sys.run();
}
