use futures_util::{SinkExt, StreamExt};
use regex::Regex;
use serde_json::{json, Value};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use warp::ws::{Message, WebSocket};
use warp::Filter;

fn get_type(message: &str) -> &str {
  if message.to_lowercase().contains("warn") {
    "warn"
  } else {
    "info"
  }
}

fn get_path_positions(message: &str) -> Vec<Value> {
  let mut positions: Vec<Value> = Vec::new();
  let path_regex = Regex::new(r"(/[^/\s]+)+").unwrap();

  for mat in path_regex.find_iter(message) {
    positions.push(json!({"start": mat.start(), "end": mat.end()}));
  }

  positions
}

fn get_json_position(message: &str) -> Vec<Value> {
  let mut positions: Vec<Value> = Vec::new();
  let mut start = None;
  let mut open_braces = 0;
  let mut close_braces = 0;

  for (i, c) in message.chars().enumerate() {
    if c == '{' {
      open_braces += 1;
      if start.is_none() {
        start = Some(i);
      }
    } else if c == '}' {
      close_braces += 1;
      if let Some(s) = start {
        if open_braces == close_braces {
          let json_str = &message[s..=i];
          if serde_json::from_str::<Value>(json_str).is_ok() {
            positions.push(json!({"start": s, "end": i}));
          }
          start = None;
          open_braces = 0;
          close_braces = 0;
        }
      }
    }
  }

  positions
}

fn read_and_log(file_path: &str, tx: broadcast::Sender<Value>, level: &str) {
  let file = File::open(file_path).expect(&format!("Error opening {}", file_path));
  let reader = BufReader::new(file);

  for line in reader.lines() {
    match line {
      Ok(log) => {
        let log_type = if level == "info" {
          get_type(&log)
        } else {
          "error"
        };
        let json_positions = get_json_position(&log);
        let log_entry = json!({
            "level": level,
            "time": SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs(),
            "msg": log,
            "type": log_type,
            "json_positions": json_positions,
            "path_positions": get_path_positions(&log),
        });

        // Notify clients about the new log entry
        let _ = tx.send(log_entry);
        println!("Log ok: {}", log);
      }
      Err(err) => eprintln!("Error reading {}: {}", file_path, err),
    }
  }
}

async fn handle_ws(ws: WebSocket, mut rx: broadcast::Receiver<Value>) {
  let (mut ws_tx, mut ws_rx) = ws.split();

  tokio::spawn(async move {
    while let Ok(log_entry) = rx.recv().await {
      if ws_tx
        .send(Message::text(log_entry.to_string()))
        .await
        .is_err()
      {
        break;
      }
    }
  });

  while let Some(result) = ws_rx.next().await {
    if result.is_err() {
      break;
    }
  }
}

#[tokio::main]
async fn main() {
  let (tx, _) = broadcast::channel(16);

  let tx_stdout = tx.clone();
  let tx_stderr = tx.clone();

  let stdout_thread = thread::spawn(move || {
    read_and_log("stdout.log", tx_stdout, "info");
  });

  let stderr_thread = thread::spawn(move || {
    read_and_log("stderr.log", tx_stderr, "error");
  });

  stdout_thread
    .join()
    .expect("The stdout thread has panicked");
  stderr_thread
    .join()
    .expect("The stderr thread has panicked");

  let ws_route = warp::path("ws")
    .and(warp::ws())
    .map(move |ws: warp::ws::Ws| {
      let rx = tx.subscribe();
      ws.on_upgrade(move |socket| handle_ws(socket, rx))
    });

  let index_route = warp::path::end().and(warp::fs::file("src/index.html"));

  let routes = ws_route.or(index_route);

  warp::serve(routes).run(([127, 0, 0, 1], 1312)).await;
}
