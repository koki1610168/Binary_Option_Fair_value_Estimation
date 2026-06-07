use tokio::time;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use futures_util::{SinkExt, StreamExt};
use url::Url;
use std::fs::OpenOptions;
use csv;
use statrs::distribution::{Normal, ContinuousCDF};

#[derive(Debug, Deserialize)]
struct Instrument {
    base_currency: String,
    creation_timestamp: u64,
    expiration_timestamp: u64,
    instrument_name: String,
    option_type: String,
    strike: f64,
}

#[derive(Debug, Deserialize)]
struct DeribitResponse {
    jsonrpc: String,
    result: Vec<Instrument>
}

#[derive(Debug, Deserialize)]
struct TickerData {
    instrument_name: String,
    mark_iv: Option<f64>,
    bid_iv: Option<f64>,
    ask_iv: Option<f64>,
    mark_price: Option<f64>,
    best_bid_price: Option<f64>,
    best_ask_price: Option<f64>,
    open_interest: Option<f64>,
    underlying_price: Option<f64>
}

#[derive(Debug, Deserialize)]
struct TickerResponse {
    result: TickerData
}



#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");


    let url = "https://www.deribit.com/api/v2/public/get_instruments";
    let ticker_url = "https://www.deribit.com/api/v2/public/ticker";

    let resp = reqwest::Client::new()
        .get(url)
        .query(&[
            ("currency", "BTC"),
            ("kind", "option"),
            ("expired", "false"),
        ])
        .send()
        .await?;

    let data: DeribitResponse = resp.json().await?;

    let client = reqwest::Client::new();

    for instrument in data.result.iter().take(20) {
        let ticker_resp = client
            .get(ticker_url)
            .query(&[
                ("instrument_name", instrument.instrument_name.as_str())
            ])
            .send()
            .await?
            .error_for_status()?;

        let ticker: TickerResponse = ticker_resp.json().await?;

        let t = ticker.result;

        if is_good_ticker(&t) {
            if instrument.option_type == "put" {

                let now_ms = chrono::Utc::now().timestamp_millis() as u64;

                let time_to_expiry_years = (instrument.expiration_timestamp - now_ms) as f64
                    /1000.0 / 60.0 /60.0 / 24.0 / 365.0;
                let fair = binary_put_fair_value(t.underlying_price.unwrap(), instrument.strike,
                    t.mark_iv.unwrap(), time_to_expiry_years, 0.0);

                println!(
                    "{} strike={} expiry={} type={} mark_iv={:?} bid={:?} ask={:?} oi={:?}",
                    instrument.instrument_name,
                    instrument.strike,
                    instrument.expiration_timestamp,
                    instrument.option_type,
                    t.mark_iv,
                    t.best_bid_price,
                    t.best_ask_price,
                    t.open_interest,
                );
                println!("{}", fair);
            }
        }


    }







    /*

    // WebSocket live option market data
    let url = Url::parse("wss://www.deribit.com/ws/api/v2")?;
    let (ws_stream, _) = connect_async(url.as_str()).await?;
    let (mut write, mut read) = ws_stream.split();

    println!("Websocket client connected");

    let message = json!(
        { 
            "jsonrpc": "2.0",
            "method": "public/subscribe",
            "params": {
                "channels": [
                    "ticker.BTC-8JUN26-50000-C.agg2"
                    ]
        
            }
        }
    );

    write.send(Message::Text(message.to_string().into())).await?;

    while let Some(msg) = read.next().await {
        let msg = msg?;

        if msg.is_text() {
            let text = msg.to_text()?;
            println!("{:#?}", text);
        }
    }

    */
    
    Ok(())
}


fn is_good_ticker(t: &TickerData) -> bool {
    let bid = match t.best_bid_price {
        Some(x) if x > 0.0 => x,
        _ => return false,
    };

    let ask = match t.best_ask_price {
        Some(x) if x > 0.0 => x,
        _ => return false,
    };

    let mark_iv = match t.mark_iv {
        Some(x) if x > 20.0 && x < 250.0 => x,
        _ => return false,
    };

    let open_interest = t.open_interest.unwrap_or(0.0);

    if open_interest == 0.0 {
        return false;
    }

    let mid = (bid + ask) / 2.0;
    let spread_pct = (ask - bid) / mid;

    if spread_pct > 0.20 {
        return false;
    }

    true
}

fn binary_put_fair_value(
    spot: f64,
    strike: f64,
    iv_percent: f64,
    time_to_expiry_years: f64,
    rate: f64,
) -> f64 {
    let sigma = iv_percent / 100.0;
    let normal = Normal::new(0.0, 1.0).unwrap();

    let d2 = ((spot / strike).ln()
        + (rate - 0.5 * sigma * sigma) * time_to_expiry_years)
        / (sigma * time_to_expiry_years.sqrt());

    normal.cdf(-d2)
}
