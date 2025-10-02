use serde::{Deserialize, Serialize};

mod commands;

pub mod client;

#[allow(dead_code)]
pub mod responses;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KalshiChannel {
    OrderbookDelta,
    Ticker,
    Trade,
    Fill,
    MarketLifecycleV2,
}

impl KalshiChannel {
    const fn as_str(&self) -> &'static str {
        match self {
            KalshiChannel::OrderbookDelta => "orderbook_delta",
            KalshiChannel::Ticker => "ticker",
            KalshiChannel::Trade => "trade",
            KalshiChannel::Fill => "fill",
            KalshiChannel::MarketLifecycleV2 => "market_lifecycle_v2",
        }
    }
}

impl Into<&'static str> for KalshiChannel {
    fn into(self) -> &'static str {
        self.as_str()
    }
}
