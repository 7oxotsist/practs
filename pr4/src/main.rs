
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use rand::distr::Alphanumeric;
use rand::{thread_rng, Rng};
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
    started: bool,
    finished: bool,
}

#[derive(Clone)]
pub struct Game {
    pub map: PeerMap,
    logic: Arc<Mutex<GameLogic>>,
}

impl GameLogic {
    fn generate_code(&mut self) {
        self.secret_code = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect::<String>()
            .to_uppercase();
    }

    pub fn new() -> Self {
        let mut logic = Self {
            secret_code: String::new(),
            max_attempts: 6,
            current_attempt: 0,
            started: false,
            finished: false,
        };
        logic.generate_code();
        logic
    }

    fn validate_guess(&mut self, guess: &str) -> Vec<String> {
        let secret: Vec<char> = self.secret_code.chars().collect();
        let guess_chars: Vec<char> = guess.chars().collect();
        let mut result = Vec::with_capacity(secret.len());

        for (secret_char, guess_char) in secret.iter().zip(guess_chars.iter()) {
            if secret_char == guess_char {
                result.push("◘".to_string());
            } else {
                result.push("_".to_string());
            }
        }
        self.current_attempt += 1;
        result
    }

    fn start_game(&mut self) {
        self.generate_code();
        println!("Игра началась! Секретный код: {}", self.secret_code);
        self.current_attempt = 0;
        self.started = true;
        self.finished = false;
    }

    fn finish_game(&mut self) {
        self.started = false;
        self.finished = true;
    }
}

impl Game {
    pub async fn add_player(&self, player: SocketAddr, sender: UnboundedSender<Message>) {
        let mut users = self.map.lock().await;
        if !users.contains_key(&player) && users.len() < 4 {
            users.insert(player, sender);
        }
    }

    pub async fn remove_player(&self, player: &SocketAddr) {
        let mut users = self.map.lock().await;
        users.remove(player);
    }

    /// Возвращает строку со списком игроков и их количеством
    pub async fn show_players(&self) -> String {
        let users = self.map.lock().await;
        if users.is_empty() {
            "Нет подключённых игроков.".to_owned()
        } else {
            let mut buff = String::new();
            buff.push_str(&format!("Список подключённых игроков ({}):\n", users.len()));
            for addr in users.keys() {
                buff.push_str(&format!("- {}\n", addr));
            }
            buff
        }
    }

    async fn broadcast(&self, message: &str) {
        println!("broadcasting: {}", message);
        let players = self.map.lock().await;
        for (_addr, sender) in players.iter() {
            let _ = sender.send(Message::Text(message.to_string().into()));
        }
    }

    pub fn broadcast_async(self: Arc<Self>, message: String) {
        tokio::spawn(async move {
            self.broadcast(&message).await;
        });
    }

    pub fn new() -> Self {
        Self {
            map: Arc::new(Mutex::new(HashMap::new())),
            logic: Arc::new(Mutex::new(GameLogic::new())),
        }
    }
}

async fn handshake(game_state: Game, stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake error: {}", e);
            return;
        }
    };
    println!("WebSocket connection established: {}", addr);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = unbounded_channel();
    game_state.add_player(addr, tx.clone()).await;

    // После добавления игрока проверяем, нужно ли стартовать игру
    {
        let arc_game = Arc::new(game_state.clone());
        let mut logic = arc_game.logic.lock().await;
        let users = arc_game.map.lock().await;
        if users.len() > 1 && !logic.started && !logic.finished {
            logic.start_game();
            let msg = format!("Игра начинается! Угадайте 5-значный код. Попыток: {}", logic.max_attempts);
            arc_game.clone().broadcast_async(msg);
        }
    }

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Основной цикл чтения сообщений
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                handle_client_message(&game_state, addr, text.to_string()).await;
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }

    // После завершения соединения (по любой причине) удаляем игрока
    game_state.remove_player(&addr).await;

    let _ = send_task.await;
}

async fn handle_client_message(game_state: &Game, addr: SocketAddr, message: String) {
    println!("Получено сообщение от {}: {}", addr, message);

    let arc_game = Arc::new(game_state.clone());

    let mut gl = arc_game.logic.lock().await;
    if !gl.started || gl.finished {
        // Игра не началась или уже закончилась
        arc_game.clone().broadcast_async("Ожидание других игроков для начала игры...".to_string());
        return;
    }

    // Проверяем, не превысил ли игрок лимит попыток
    if gl.current_attempt >= gl.max_attempts {
        arc_game.clone().broadcast_async(format!(
            "Попытки закончились! Никто не угадал. Игра окончена."
        ));
        let code = gl.secret_code.clone();
        gl.finish_game();
        drop(gl); // Освобождаем lock перед стартом нового раунда
        start_new_round(arc_game, code).await;
        return;
    }

    let guess = message.trim().to_uppercase();
    if guess.len() != gl.secret_code.len() {
        arc_game.clone().broadcast_async(format!(
            "Неверная длина кода! Введите {} символов.", gl.secret_code.len()
        ));
        return;
    }

    let result = gl.validate_guess(&guess);
    let result_str = result.join("");
    arc_game.clone().broadcast_async(format!("{}: {}", guess, result_str));

    if guess == gl.secret_code {
        arc_game.clone().broadcast_async(format!(
            "Игрок {} угадал код! Победа! Игра окончена.",
            addr
        ));
        let code = gl.secret_code.clone();
        gl.finish_game();
        drop(gl); // Освобождаем lock перед стартом нового раунда
        start_new_round(arc_game, code).await;
    } else if gl.current_attempt >= gl.max_attempts {
        arc_game.clone().broadcast_async(format!(
            "Попытки закончились! Никто не угадал. Игра окончена."
        ));
        let code = gl.secret_code.clone();
        gl.finish_game();
        drop(gl);
        start_new_round(arc_game, code).await;
    }
}

async fn start_new_round(game: Arc<Game>, prev_code: String) {
    // Сообщаем игрокам правильный код прошлого раунда
    game.clone().broadcast_async(format!(
        "Правильный код прошлого раунда был: {}. Новый раунд начинается!", prev_code
    ));
    // Стартуем новый раунд
    let mut gl = game.logic.lock().await;
    gl.start_game();
    let msg = format!("Игра начинается! Угадайте 5-значный код. Попыток: {}", gl.max_attempts);
    drop(gl);
    game.clone().broadcast_async(msg);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8090").await?;
    println!("Listening on {}", listener.local_addr().unwrap());
    let game = Game::new();

    loop {
        let (stream, addr) = listener.accept().await?;
        let state = game.clone();
        tokio::spawn(async move {
            handshake(state, stream, addr).await;
        });
    }
}
