#![allow(unused)]

use futures_util::{select_biased, FutureExt, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{error::Error, time::Duration, vec};
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
    time::{interval, MissedTickBehavior},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{http::Request, Message},
    MaybeTlsStream, WebSocketStream,
};

use crate::Kalshi;

pub struct KalshiWebsocketClient {
    _ws: JoinHandle<()>,
    next_cmd_id: u32,
    to_kalshi: UnboundedSender<KalshiCommand>,
    pub stream: UnboundedReceiver<KalshiWebsocketResponse>,
}

async fn kalshi_ws_handler(
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    from_kalshi_tx: UnboundedSender<KalshiWebsocketResponse>,
    mut to_kalshi_rx: UnboundedReceiver<KalshiCommand>,
) {
    let mut stream = Box::pin(stream.fuse());
    let mut heartbeat = interval(Duration::from_secs(10));
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        select_biased! {
            cmd = to_kalshi_rx.recv().fuse() => {
                match cmd {
                    Some(cmd) => {
                        stream.send(Message::Text(serde_json::to_string(&cmd).unwrap())).await.unwrap();
                    },
                    _ => panic!("All senders dropped")
                }
            }
            _ = heartbeat.tick().fuse() => {
                stream.send(Message::Ping(vec![])).await.expect("Expected ping to be received");
            }
            msg = stream.select_next_some() => {
                let msg = msg.unwrap();
                match msg {
                    Message::Text(text) => {
                        let res: KalshiWebsocketResponse = serde_json::from_str(&text).unwrap();
                        from_kalshi_tx.send(res).unwrap();
                    },
                    Message::Ping(inner) => stream.send(Message::Pong(inner)).await.expect("Expected pong to be received"),
                    Message::Close(_) => stream.close().await.expect("Stream failed to close"),
                    _ => {},
                }
            }
        }
    }
}

impl KalshiWebsocketClient {
    pub async fn connect(kalshi: &Kalshi) -> Result<Self, Box<dyn Error>> {
        let curr_token = kalshi
            .get_user_token()
            .ok_or("No user token, login first using .login(..)".to_string())?;
        let req = Request::builder()
            .uri(kalshi.get_base_url())
            .header("Authorization:Bearer", curr_token)
            .body(())?;
        let (ws_stream, _) = connect_async(req).await?;

        let (to_kalshi_tx, to_kalshi_rx) = unbounded_channel::<KalshiCommand>();
        let (from_kalshi_tx, from_kalshi_rx) = unbounded_channel::<KalshiWebsocketResponse>();

        let _ws = tokio::spawn(kalshi_ws_handler(ws_stream, from_kalshi_tx, to_kalshi_rx));

        Ok(KalshiWebsocketClient {
            next_cmd_id: 0,
            to_kalshi: to_kalshi_tx,
            stream: from_kalshi_rx,
            _ws,
        })
    }

    pub async fn subscribe(
        &mut self,
        channels: Vec<KalshiChannel>,
        market_tickers: Vec<String>,
    ) -> Result<u32, Box<dyn Error>> {
        let cmd_id = self.next_cmd_id;
        if channels.contains(&KalshiChannel::Orderbook) && market_tickers.len() == 0 {
            return Err("Cannot subscribe to orderbooks for all market tickers, provide at least one market ticker".to_string().into());
        }
        let msg = KalshiCommand::Subscribe {
            id: cmd_id,
            cmd: "subscribe".to_string(),
            params: KalshiSubscribeCommandParams {
                channels,
                market_tickers,
            },
        };
        self.to_kalshi.send(msg)?;
        self.next_cmd_id += 1;
        Ok(cmd_id)
    }

    pub async fn close(self) -> Result<(), Box<dyn Error>> {
        self.to_kalshi.send(KalshiCommand::End)?;
        Ok(())
    }
}

impl Drop for KalshiWebsocketClient {
    fn drop(&mut self) {
        self.to_kalshi
            .send(KalshiCommand::End)
            .expect("Tried to close kalshi websocket connection on drop but got error");
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum KalshiWebsocketResponseMessage {
    OrderbookSnapshot {
        market_ticker: String,
        yes: Option<Vec<(u32, i32)>>,
        no: Option<Vec<(u32, i32)>>,
    },
    OrderbookDelta {
        market_ticker: String,
        delta: i32,
        price: u32,
        side: String,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct KalshiWebsocketResponse {
    pub sid: u32,
    #[serde(rename = "type")]
    pub response_type: String,
    pub seq: u32,
    pub msg: KalshiWebsocketResponseMessage,
}

#[derive(Serialize, Clone, Debug)]
pub enum KalshiCommand {
    Subscribe {
        id: u32,
        cmd: String,
        params: KalshiSubscribeCommandParams,
    },
    // UpdateSubscription
    // Unsubscribe
    End, // Not a Kalshi command but signals to send prob messaging to close the ws stream
}
#[derive(Serialize, Clone, Debug)]
pub struct KalshiSubscribeCommandParams {
    channels: Vec<KalshiChannel>,
    market_tickers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KalshiChannel {
    Orderbook,
    Ticker,
    Trade,
    Fill,
    MarketLifecycle,
}

impl KalshiChannel {
    const fn as_str(&self) -> &'static str {
        match self {
            KalshiChannel::Orderbook => "orderbook_delta",
            KalshiChannel::Ticker => "ticker",
            KalshiChannel::Trade => "trade",
            KalshiChannel::Fill => "fill",
            KalshiChannel::MarketLifecycle => "market_lifecycle",
        }
    }
}

impl Into<&'static str> for KalshiChannel {
    fn into(self) -> &'static str {
        self.as_str()
    }
}
