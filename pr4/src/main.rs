use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc};
use futures_util::StreamExt;
use rand;
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::handshake::client::generate_key;
use tokio_tungstenite::tungstenite::protocol::Message;

type PeerMap = Arc<Mutex<HashMap<SocketAddr, UnboundedSender<Message>>>>;

#[derive(Clone)]
struct GameLogic {
    secret_code: String,
    max_attempts: u8,
    current_attempt: u8,
}

#[derive(Clone)]
pub struct Game {
    pub map: PeerMap,
    logic: Arc<Mutex<GameLogic>>
}

impl GameLogic {
     fn generate_code(&mut self) {
        self.secret_code = rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(|c| c as char)
            .collect::<String>()
            .to_uppercase();
    }

    pub fn new() -> Self {
         Self {
            secret_code: generate_key(),
            max_attempts: 6,
            current_attempt: 0
        }
    }

     fn validate_guess(mut self, guess: &str) -> Vec<String> {
        let secret: Vec<char> = self.secret_code.chars().collect();
        let guess_chars: Vec<char> = guess.chars().collect();
        let mut result = Vec::with_capacity(secret.len());

        for (i, (secret_char, guess_char)) in secret.iter().zip(guess_chars.iter()).enumerate() {
            if secret_char == guess_char {
                result.push("⬜".to_string());
            } else {
               result.push("⬛".to_string());
            }
        }
        self.current_attempt += 1;
        result
    }

    fn start_game(&mut self) {
        println!("Игра началась! Секретный код: {}", self.secret_code);
        self.current_attempt = 0;
    }
}

impl Game {
    pub async fn add_player(&self, player: SocketAddr, receiver: UnboundedSender<Message>) {
        let mut users:tokio::sync::MutexGuard<'_, HashMap<SocketAddr, UnboundedSender<Message>>> = self.map.lock().await;
        if !users.contains_key(&player) && users.len() <= 4 {
            users.insert(player, receiver);
        }
    }

    async fn broadcast(&self, message: &str) {
        let players = self.map.lock().await;
        for (addr, sender) in players.iter() {
            let _ = sender.send(Message::Text(message.to_string().into()));
        }
    }

    pub fn new() -> Self {
        Self {
            map: Arc::new(Mutex::new(HashMap::new())),
            logic: Arc::new(Mutex::new(GameLogic::new()))
        }
    }
}

async fn handshake(game_state: Game, stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, rx) = unbounded_channel();
    game_state.add_player(addr, tx.clone());

    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                handle_client_message(&game_state, addr, text.to_string()).await;
            }
             Message::Close(_) => break,
             _ => {}
        }
    }
}

async fn handle_client_message(game_state: &Game, addr: SocketAddr, message: String) {
    println!("Получено сообщение от {}: {}", addr, message);
    
    if message == "/READY" {
        game_state.broadcast("Игра начинается!").await;
        let mut gl = game_state.logic.lock().await;
        
        let gs = game_state.map.lock().await;
        if gs.len() > 1 {
            gl.start_game();
        }
    } else {
        let gl = game_state.logic.lock().await;
        let result = gl.clone().validate_guess(&message.to_uppercase());
        let result_str = result.join("");
        game_state.broadcast(&format!("{}: {}", message, result_str)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8090").await?;
    println!("Listening on {}", listener.local_addr().unwrap());
    //let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<>();
    let game = Game::new();
     
    loop {
        let (stream, addr) = listener.accept().await?;
        let state = game.clone();
        tokio::spawn(async move {
            handshake(state, stream, addr).await;
        });
    }
}
