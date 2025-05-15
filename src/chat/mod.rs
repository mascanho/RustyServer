use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ---- Actor Messages ----
#[derive(Message)]
#[rtype(result = "()")]
pub struct ChatMessage(pub String); // Incoming message from client

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: usize,
    pub addr: actix::Addr<WsChatSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

// ---- WebSocket Session (per connection) ----
pub struct WsChatSession {
    pub id: usize,
    pub server: actix::Addr<ChatServer>,
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                self.server.do_send(ChatMessage(text.to_string()));
            }
            Ok(ws::Message::Close(_)) => {
                self.server.do_send(Disconnect { id: self.id });
                ctx.stop();
            }
            _ => (),
        }
    }
}

// ---- Chat Server (Global State) ----
pub struct ChatServer {
    pub sessions: Arc<Mutex<HashMap<usize, actix::Addr<WsChatSession>>>>,
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Actor for ChatServer {
    type Context = actix::Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) {
        self.sessions.lock().unwrap().insert(msg.id, msg.addr);
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        self.sessions.lock().unwrap().remove(&msg.id);
    }
}

impl Handler<ChatMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, _: &mut Self::Context) {
        for (_, addr) in self.sessions.lock().unwrap().iter() {
            addr.do_send(ws::Message::Text(msg.0.clone()));
        }
    }
}

// ---- HTTP Handler ----
pub async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<actix::Addr<ChatServer>>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        WsChatSession {
            id: rand::random::<usize>(),
            server: server.get_ref().clone(),
        },
        &req,
        stream,
    )?;
    Ok(resp)
}
