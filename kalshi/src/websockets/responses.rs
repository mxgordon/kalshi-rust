use serde::Deserialize;

use super::KalshiChannel;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum KalshiWebsocketResponse {
    OrderbookSnapshot {
        sid: u32,
        seq: u32,
        msg: KalshiOrderbookSnapshotMessage,
    },
    OrderbookDelta {
        sid: u32,
        seq: u32,
        msg: KalshiOrderbookDeltaMessage,
    },
    Ticker {
        sid: u32,
        msg: KalshiTickerMessage,
    },
    Trade {
        sid: u32,
        msg: KalshiTradeMessage,
    },
    Fill {
        sid: u32,
        msg: KalshiFillMessage,
    },
    EventLifecycle {
        sid: u32,
        msg: KalshiEventLifecycleMessage,
    },
    MarketLifecycleV2 {
        sid: u32,
        msg: KalshiMarketLifecycleMessage,
    },
    Subscribed {
        msg: KalshiOrderbookSubscribedMessage,
    },
    Error {
        id: u32,
        msg: KalshiOrderbookErrorMessage,
    },
    Ok {
        id: u32,
        sid: u32,
        seq: u32,
        market_tickers: Vec<String>,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct KalshiOrderbookSubscribedMessage {
    channel: KalshiChannel,
    sid: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct KalshiOrderbookErrorMessage {
    code: u32,
    msg: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct KalshiOrderbookSnapshotMessage {
    market_ticker: String,
    yes: Option<Vec<(u32, i32)>>,
    no: Option<Vec<(u32, i32)>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct KalshiOrderbookDeltaMessage {
    delta: i32,
    price: u32,
    side: String,
    client_order_id: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct KalshiTickerMessage {
    market_ticker: String,
    price: u32,
    yes_bid: u32,
    yes_ask: u32,
    volume: u32,
    open_interest: u32,
    dollar_volume: u32,
    dollar_open_interest: u32,
    ts: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct KalshiTradeMessage {
    pub market_ticker: String,
    pub yes_price: u32,
    pub no_price: u32,
    pub count: u32,
    pub taker_side: KalshiSide,
    pub ts: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct KalshiFillMessage {
    trade_id: String,
    order_id: String,
    market_ticker: String,
    is_taker: bool,
    side: KalshiSide,
    yes_price: u32,
    no_price: u32,
    count: u32,
    action: String,
    ts: u32,
    client_order_id: Option<String>,
    post_position: u32,
    purchased_side: KalshiSide,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "event_type")]
#[serde(rename_all = "snake_case")]
pub enum KalshiMarketLifecycleMessage {
    Created {
        market_ticker: String,
        open_ts: u32,
        close_ts: u32,
        additional_metadata: MarketLifecycleAdditionalMetadata,
    },
    Activated {
        market_ticker: String,
        is_deactivated: bool,
    },
    Deactivated {
        market_ticker: String,
        is_deactivated: bool,
    },
    CloseDateUpdated {
        market_ticker: String,
        close_ts: u32,
    },
    Determined {
        market_ticker: String,
        result: String,
        determination_ts: u32,
    },
    Settled {
        market_ticker: String,
        settled_ts: u32,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct MarketLifecycleAdditionalMetadata {
    pub name: String,
    pub title: String,
    pub yes_sub_title: String,
    pub no_sub_title: String,
    pub rules_primary: String,
    pub rules_secondary: String,
    pub can_close_early: bool,
    #[serde(default)]
    pub event_ticker: Option<String>,
    pub expected_expiration_ts: u32,
    #[serde(default)]
    pub strike_type: Option<String>,
    #[serde(default)]
    pub floor_strike: Option<f64>,
    #[serde(default)]
    pub cap_strike: Option<bool>,
    #[serde(default)]
    pub custom_strike: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct KalshiEventLifecycleMessage {
    event_ticker: String,
    title: String,
    subtitle: String,
    collateral_return_type: String,
    series_ticker: String,
    strike_date: Option<u32>,
    strike_period: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum KalshiSide {
    Yes,
    No,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum KalshiAction {
    Buy,
    Sell,
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_event_lifecycle_deserialization() {
        let raw = r#"{"type":"event_lifecycle","sid":1,"msg":{"event_ticker":"KXTECHLAYOFF-25SEP","title":"Tech layoffs up in Sep 2025?","subtitle":"","collateral_return_type":"","series_ticker":""}}"#;
        let parsed = serde_json::from_str::<KalshiWebsocketResponse>(raw);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            KalshiWebsocketResponse::EventLifecycle { sid, msg } => {
                assert_eq!(sid, 1);
                assert_eq!(msg.event_ticker, "KXTECHLAYOFF-25SEP");
                assert_eq!(msg.title, "Tech layoffs up in Sep 2025?");
            }
            _ => panic!("Expected EventLifecycle variant"),
        }
    }

    #[test]
    fn test_market_lifecycle_determined() {
        let raw = r#"{"type":"market_lifecycle_v2","sid":1,"msg":{"market_ticker":"KXMLBGAME-25OCT01DETCLE-DET","determination_ts":1759350638,"result":"no","event_type":"determined"}}"#;
        let parsed = serde_json::from_str::<KalshiWebsocketResponse>(raw);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            KalshiWebsocketResponse::MarketLifecycleV2 { sid, msg } => {
                assert_eq!(sid, 1);
                match msg {
                    KalshiMarketLifecycleMessage::Determined {
                        market_ticker,
                        result,
                        determination_ts,
                    } => {
                        assert_eq!(market_ticker, "KXMLBGAME-25OCT01DETCLE-DET");
                        assert_eq!(result, "no");
                        assert_eq!(determination_ts, 1759350638);
                    }
                    _ => panic!("Expected Determined variant"),
                }
            }
            _ => panic!("Expected MarketLifecycleV2 variant"),
        }
    }
    #[test]
    fn test_market_lifecycle_settled() {
        let raw = r#"{"type":"market_lifecycle_v2","sid":1,"seq":1,"msg":{"market_ticker":"KXMLBTOTAL-25OCT01DETCLE-5","settled_ts":1759351985,"event_type":"settled"}}"#;
        let parsed = serde_json::from_str::<KalshiWebsocketResponse>(raw);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            KalshiWebsocketResponse::MarketLifecycleV2 { sid, msg } => {
                assert_eq!(sid, 1);
                match msg {
                    KalshiMarketLifecycleMessage::Settled {
                        market_ticker,
                        settled_ts,
                    } => {
                        assert_eq!(market_ticker, "KXMLBTOTAL-25OCT01DETCLE-5");
                        assert_eq!(settled_ts, 1759351985);
                    }
                    _ => panic!("Expected Settled variant"),
                }
            }
            _ => panic!("Expected MarketLifecycleV2 variant"),
        }
    }

    #[test]
    fn test_market_lifecycle_close_date_updated() {
        let raw = r#"{"type":"market_lifecycle_v2","sid":1,"seq":43,"msg":{"market_ticker":"KXMLBWINS-NYM-25-T80","close_ts":1759353839,"event_type":"close_date_updated"}}"#;
        let parsed = serde_json::from_str::<KalshiWebsocketResponse>(raw);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            KalshiWebsocketResponse::MarketLifecycleV2 { sid, msg } => {
                assert_eq!(sid, 1);
                match msg {
                    KalshiMarketLifecycleMessage::CloseDateUpdated {
                        market_ticker,
                        close_ts,
                    } => {
                        assert_eq!(market_ticker, "KXMLBWINS-NYM-25-T80");
                        assert_eq!(close_ts, 1759353839);
                    }
                    _ => panic!("Expected CloseDateUpdated variant"),
                }
            }
            _ => panic!("Expected MarketLifecycleV2 variant"),
        }
    }

    #[test]
    fn test_market_lifecycle_deactivated() {
        let raw = r#"{"type":"market_lifecycle_v2","sid":1,"seq":7,"msg":{"market_ticker":"KXNFLFIRSTTD-25OCT05MIACAR-CARCHUBBARD30","is_deactivated":true,"event_type":"deactivated"}}"#;
        let parsed = serde_json::from_str::<KalshiWebsocketResponse>(raw);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            KalshiWebsocketResponse::MarketLifecycleV2 { sid, msg } => {
                assert_eq!(sid, 1);
                match msg {
                    KalshiMarketLifecycleMessage::Deactivated {
                        market_ticker,
                        is_deactivated,
                    } => {
                        assert_eq!(market_ticker, "KXNFLFIRSTTD-25OCT05MIACAR-CARCHUBBARD30");
                        assert_eq!(is_deactivated, true);
                    }
                    _ => panic!("Expected Deactivated variant"),
                }
            }
            _ => panic!("Expected MarketLifecycleV2 variant"),
        }
    }

    #[rstest::rstest]
    #[case("KXWTAMATCH-25OCT02LYSGAU-LYS", 1759352700, 1760598000, r#"{"type":"market_lifecycle_v2","sid":1,"seq":29,"msg":{"market_ticker":"KXWTAMATCH-25OCT02LYSGAU-LYS","open_ts":1759352700,"close_ts":1760598000,"additional_metadata":{"name":"Eva Lys","title":"Will Eva Lys win the Lys vs Gauff match?","yes_sub_title":"Eva Lys","no_sub_title":"Eva Lys","rules_primary":"If Eva Lys wins the Lys vs Gauff professional tennis match in the 2025 WTA Beijing quarterfinal after a ball has been played, then the market resolves to Yes.","rules_secondary":"The following market refers to the Lys vs Gauff professional tennis match in the 2025 WTA Beijing quarterfinal after a ball has been played. If the match does not occur (signaled by a ball being played) due to a player injury, walkover, forfeiture, or any other cancellation (all before the match starts), the market will resolve to a fair price in accordance with the rules. If this match is postponed or delayed, the market will remain open and close after the rescheduled match has finished (within two weeks).","can_close_early":true,"event_ticker":"KXWTAMATCH-25OCT02LYSGAU","expected_expiration_ts":1759399200,"strike_type":"structured","custom_strike":{"tennis_competitor":"6c69f42c-5e27-4cfd-a9ea-8eb9fbb0bd12"}},"event_type":"created"}}"#)]
    #[case("KXSWIFTATTENDEVENT-25OCT04", 1759375800, 1759672800, r#"{"type":"market_lifecycle_v2","sid":1,"seq":1239,"msg":{"market_ticker":"KXSWIFTATTENDEVENT-25OCT04","open_ts":1759375800,"close_ts":1759672800,"additional_metadata":{"name":"Yes","title":"Will Taylor Swift attend Saturday Night Live Season 51 Premiere?","yes_sub_title":"Yes","no_sub_title":"Yes","rules_primary":"If Taylor Swift attends Saturday Night Live Season 51 Premiere, then the market resolves to Yes.","rules_secondary":"Attendance is confirmed if the person is reported present at the event by any Source Agency, including social media posts by the person themselves. Virtual attendance counts as attendance unless the event is explicitly in-person only. Brief appearances or partial attendance count as attendance. The event must occur in the specified year. If the event is cancelled, postponed beyond the expiration date, or does not occur in the specified year, the market resolves to No.","can_close_early":true,"event_ticker":"KXSWIFTATTENDEVENT-25OCT04","expected_expiration_ts":1759672800},"event_type":"created"}}"#)]
    #[case("KXFDVLIGHTER-25DEC31-8", 1759521600, 1767200340, r#"{"type":"market_lifecycle_v2","sid":1,"seq":956,"msg":{"market_ticker":"KXFDVLIGHTER-25DEC31-8","open_ts":1759521600,"close_ts":1767200340,"additional_metadata":{"name":"$8,000,000,000+","title":"What will Lighter FDV be at 10:00 AM ET 1 day after launch?","yes_sub_title":"$8,000,000,000+","no_sub_title":"$8,000,000,000+","rules_primary":"If Lighter (X)'s fully diluted valuation (FDV) as displayed on CoinGecko is above 7999999999.99 at exactly 10:00 AM on the date of the launch, then the market resolves to Yes.","rules_secondary":"The FDV must be the value shown in the \"FDV\" field on the coin's main CoinGecko page, not calculated from other metrics. If no data is available at the specified time, the last FDV value shown before that time on the date will be used. If the coin is delisted from CoinGecko before the measurement date, the last recorded FDV will be used and the market will resolve immediately. Token migrations to new smart contract addresses are tracked continuously. If CoinGecko changes its FDV calculation methodology, the value displayed at the measurement time under the FDV label will be used regardless. Post-redenomination FDV values (after splits/reverse splits) will be used. All values are in USD as shown on CoinGecko. Values can be expressed in millions (M), billions (B), or trillions (T) format.  If Lighter doesn't launch a token by December 31, 2025, 11:59 PM ET, this market will resolve to \"No.\"","can_close_early":true,"event_ticker":"KXFDVLIGHTER-25DEC31","expected_expiration_ts":1767200400,"strike_type":"greater","floor_strike":7999999999.99},"event_type":"created"}}"#)]
    fn test_market_lifecycle_created(
        #[case] expected_ticker: &str,
        #[case] expected_open_ts: u32,
        #[case] expected_close_ts: u32,
        #[case] raw: &str,
    ) {
        let parsed = serde_json::from_str::<KalshiWebsocketResponse>(raw);
        if let Err(e) = &parsed {
            eprintln!("Parse error: {}", e);
        }
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            KalshiWebsocketResponse::MarketLifecycleV2 { sid, msg } => {
                assert_eq!(sid, 1);
                match msg {
                    KalshiMarketLifecycleMessage::Created {
                        market_ticker,
                        open_ts,
                        close_ts,
                        ..
                    } => {
                        assert_eq!(market_ticker, expected_ticker);
                        assert_eq!(open_ts, expected_open_ts);
                        assert_eq!(close_ts, expected_close_ts);
                    }
                    _ => panic!("Expected Created variant"),
                }
            }
            _ => panic!("Expected MarketLifecycleV2 variant"),
        }
    }

    #[test]
    fn test_trade_message() {
        let raw = r#"{"type":"trade","sid":1,"seq":16,"msg":{"trade_id":"5b0276ef-7715-46f2-56d8-a1c7b9e59e58","market_ticker":"KXHIGHCHI-25OCT02-B80.5","yes_price":27,"no_price":73,"yes_price_dollars":"0.2700","no_price_dollars":"0.7300","count":7,"taker_side":"yes","ts":1759350609}}"#;
        let parsed = serde_json::from_str::<KalshiWebsocketResponse>(raw);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            KalshiWebsocketResponse::Trade { sid, msg } => {
                assert_eq!(sid, 1);
                assert_eq!(msg.market_ticker, "KXHIGHCHI-25OCT02-B80.5");
                assert_eq!(msg.yes_price, 27);
                assert_eq!(msg.no_price, 73);
                assert_eq!(msg.count, 7);
                assert_eq!(msg.ts, 1759350609);
                match msg.taker_side {
                    KalshiSide::Yes => {}
                    _ => panic!("Expected Yes side"),
                }
            }
            _ => panic!("Expected Trade variant"),
        }
    }

    #[test]
    fn test_ticker_message() {
        let raw = r#"{"type":"ticker","sid":1,"msg":{"market_id":"4ec1095a-e401-4898-b951-e5e9876d0afd","market_ticker":"KXNFLFIRSTTD-25OCT05NYGNO-NOSRATTLER2","price":0,"yes_bid":0,"yes_ask":3,"price_dollars":"","yes_bid_dollars":"0.0000","yes_ask_dollars":"0.0300","volume":0,"open_interest":0,"dollar_volume":0,"dollar_open_interest":0,"ts":1759351915,"Clock":4117569085}}"#;
        let parsed = serde_json::from_str::<KalshiWebsocketResponse>(raw);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            KalshiWebsocketResponse::Ticker { sid, msg } => {
                assert_eq!(sid, 1);
                assert_eq!(msg.market_ticker, "KXNFLFIRSTTD-25OCT05NYGNO-NOSRATTLER2");
                assert_eq!(msg.price, 0);
                assert_eq!(msg.yes_bid, 0);
                assert_eq!(msg.yes_ask, 3);
                assert_eq!(msg.volume, 0);
                assert_eq!(msg.open_interest, 0);
                assert_eq!(msg.dollar_volume, 0);
                assert_eq!(msg.dollar_open_interest, 0);
                assert_eq!(msg.ts, 1759351915);
            }
            _ => panic!("Expected Ticker variant"),
        }
    }
}
