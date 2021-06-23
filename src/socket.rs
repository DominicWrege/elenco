use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_broker::BrokerSubscribe;
use actix_web_actors::ws;
use std::time::{Duration, Instant};
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


pub struct LiveFeedSocket {
    heart_beat: Instant,
}

impl Actor for LiveFeedSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // self.subscribe_system_async::<TestJson>(ctx);
        self.subscribe_system_async::<MessageRowHtml>(ctx);
        self.heart_beat_start(ctx);
        ctx.set_mailbox_capacity(24);
        // ctx.run_interval(Duration::from_secs(3), |act, context| {
        //     context.notify(TestJson {
        //         name: "dsa".into(),
        //         xz: 11,
        //     })
        // });
    }

}

impl LiveFeedSocket {
    pub fn new() -> Self {
        Self {
            heart_beat: Instant::now(),
        }
    }
    fn heart_beat_start(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.heart_beat) > CLIENT_TIMEOUT {
                ctx.stop();

                return;
            }

            ctx.ping(b"");
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for LiveFeedSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_msg)) => {
                self.heart_beat = Instant::now();
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Err(_err) => {
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl Handler<MessageRowHtml> for LiveFeedSocket {
    type Result = ();

    fn handle(&mut self, msg: MessageRowHtml, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.value);
    }
}

#[derive(Message, Clone, serde::Serialize, Debug)]
#[rtype(result = "()")]
struct TestJson {
    name: String,
    xz: u32,
}

#[derive(Message, Clone, serde::Serialize, Debug)]
#[rtype(result = "()")]
pub struct MessageRowHtml {
    value: String,
}

impl MessageRowHtml {
    pub fn new(html: String) -> Self {
        Self { value: html }
    }
}
