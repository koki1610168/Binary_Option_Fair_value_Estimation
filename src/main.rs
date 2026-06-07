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

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");


    let url = "https://www.deribit.com/api/v2/public/get_instruments";

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


    println!("{:#?}", data);

    /*
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
                    "deribit_price_index.btc_usd",
                    "deribit_volatility_index.btc_usd"]
        
            }
        }
    );

    write.send(Message::Text(message.to_string().into())).await?;

    while let Some(msg) = read.next().await {
        let msg = msg?;

        if msg.is_text() {
            let text = msg.to_text()?;
            println!("{:?}", text);
        }
    }

    */
    
    Ok(())
}
