use assert_json_diff::assert_json_eq;
use clap::Parser;
use cuprate_zmq_types::json_message_types::*;
use serde::Deserialize;
use serde_json::Value;

use crate::MessageType::*;
const DEFAULT_ENDPOINT: &str = "tcp://127.0.0.1:18084";

#[derive(Parser, Debug, Default)]
#[command(version)]
struct Args {
    /// The endpoint to connect to
    #[arg(long, default_value(DEFAULT_ENDPOINT), env = "ZMQ_PUB")]
    endpoint: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[expect(clippy::enum_variant_names)]
enum MessageType {
    JsonMinimalChainMain,
    JsonMinimalTxPoolAdd,
    JsonFullChainMain,
    JsonFullTxPoolAdd,
    JsonFullMinerData,
}

impl MessageType {
    fn from_str(s: &str) -> Option<MessageType> {
        match s {
            "json-minimal-chain_main" => Some(JsonMinimalChainMain),
            "json-minimal-txpool_add" => Some(JsonMinimalTxPoolAdd),
            "json-full-chain_main" => Some(JsonFullChainMain),
            "json-full-txpool_add" => Some(JsonFullTxPoolAdd),
            "json-full-miner_data" => Some(JsonFullMinerData),
            _ => None,
        }
    }
}

fn validate_serialization<T: for<'a> Deserialize<'a> + serde::Serialize>(zmq_json_str: String) {
    let zmq_json_obj: Value = serde_json::from_str(&zmq_json_str).unwrap();
    let deserialized_obj: T = serde_json::from_str(&zmq_json_str).unwrap();
    let serialized_json_value: Value = serde_json::to_value(&deserialized_obj).unwrap();
    assert_json_eq!(zmq_json_obj, serialized_json_value);
}

fn format_json(input: &str) -> String {
    let parsed = match serde_json::from_str::<Value>(input) {
        Ok(parsed) => parsed,
        Err(e) => return format!("Error parsing JSON: {}", e),
    };

    serde_json::to_string_pretty(&parsed)
        .unwrap_or_else(|e| format!("Error formatting JSON: {}", e))
}

fn main() {
    let args = Args::parse();
    let endpoint = args.endpoint;

    println!("Connecting to {}", endpoint);

    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();

    assert!(subscriber.connect(&endpoint).is_ok());

    subscriber.set_subscribe(b"").unwrap(); // subscribe to everything
    println!("Subscribed to all messages");

    loop {
        let mut msg = zmq::Message::new();
        subscriber.recv(&mut msg, 0).unwrap();
        let mut msg_parts = msg.as_str().unwrap().splitn(2, ':');
        let msg_type = msg_parts.next().unwrap_or("").to_string();
        let msg_json_body = msg_parts.next().unwrap_or("").to_string();

        println!("Received zmq message type: {}", msg_type);
        let formatted_json = format_json(&msg_json_body);
        println!("{}\n", formatted_json);

        match MessageType::from_str(&msg_type) {
            Some(JsonFullChainMain) => validate_serialization::<Vec<ChainMain>>(msg_json_body),
            Some(JsonMinimalChainMain) => validate_serialization::<ChainMainMin>(msg_json_body),
            Some(JsonFullTxPoolAdd) => validate_serialization::<Vec<TxPoolAdd>>(msg_json_body),
            Some(JsonMinimalTxPoolAdd) => {
                validate_serialization::<Vec<TxPoolAddMin>>(msg_json_body)
            }
            Some(JsonFullMinerData) => validate_serialization::<MinerData>(msg_json_body),
            None => {
                println!("Received unknown message type: {}", msg_type);
                println!("Message body: {}", msg_json_body);
            }
        }
    }
}
