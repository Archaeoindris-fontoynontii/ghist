extern crate nalgebra as na;
extern crate ncollide2d;

extern crate rand;

#[macro_use]
extern crate serde_derive;

extern crate serde;
#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate actix;
extern crate actix_web;

use na::Vector2;

use actix::*;
use actix_web::server::HttpServer;
use actix_web::*;

mod server;
use server::{ClientMessage, ServerMessage};

/// This is our WebSocket route state, this state is shared with all route instances
/// via `HttpContext::state()`
struct WsGameSessionState {
    addr: Addr<server::GameServer>,
}

/// Entry point for our route
fn game_route(req: &HttpRequest<WsGameSessionState>) -> Result<HttpResponse> {
    ws::start(req, WsGameSession { id: 0, name: None })
}

struct WsGameSession {
    /// unique session id
    id: usize,
    /// peer name
    name: Option<String>,
}

impl Actor for WsGameSession {
    type Context = ws::WebsocketContext<Self, WsGameSessionState>;

    /// Method is called on actor start.
    /// We register ws session with GameServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // register self in game server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsGameSessionState, state is shared across all
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
                    Ok(res) => {
                        act.id = res;
                        ctx.text(
                            json!({
                                "id": act.id,
                                "pos": Vector2::new(400.0, 400.0),
                            }).to_string(),
                        );
                    }
                    // something is wrong with game server
                    _ => ctx.stop(),
                }
                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // notify game server
        ctx.state().addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from game server, we simply send it to peer WebSocket
impl Handler<server::Message> for WsGameSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<ws::Message, ws::ProtocolError> for WsGameSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Pong(_) => println!("Ping"),
            ws::Message::Text(text) => {
                // All the client sends are key messages so we assume that the message is a key message
                let m: ClientMessage = serde_json::from_str(text.trim()).unwrap();
                if let ClientMessage::Name(s) = &m {
                    self.name = Some(s.to_string());
                }
                // send message to game server
                ctx.state().addr.do_send(ServerMessage { id: self.id, m });
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

    // Start game server actor in separate thread
    let server: Addr<_> = Arbiter::start(|_| server::GameServer::new());

    // Create Http server with WebSocket support
    HttpServer::new(move || {
        // WebSocket sessions state
        let state = WsGameSessionState {
            addr: server.clone(),
        };

        App::with_state(state)
                .resource("/ws/", |r| r.f(game_route))
                // static resources
                .handler("/", fs::StaticFiles::new("static/").unwrap().index_file("index.html"))
    }).bind("0.0.0.0:8080")
        .unwrap()
        .start();

    println!("Started http server: http://localhost:8080");
    let _ = sys.run();
}
