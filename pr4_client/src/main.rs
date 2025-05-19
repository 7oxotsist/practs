use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use std::io::{self, BufRead, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Подключение к серверу
    let (ws_stream, _) = connect_async("ws://127.0.0.1:8090").await?;
    let (mut write, mut read) = ws_stream.split();

    println!("Connected to server. Type '/ready' to start or 5-letter words to guess");
    print!("> ");
    // Основной цикл обработки ввода
    loop {
        io::stdout().flush()?; // Синхронный ввод
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_uppercase();

        if input.is_empty() {
            continue;
        }

        // Отправка сообщения
        write.send(Message::Text(input.clone().into())).await?;

        // Получение ответа
        if let Some(msg) = read.next().await {
            match msg? {
                Message::Text(text) => println!("Server: {}", text),
                Message::Close(_) => {
                    println!("Connection closed");
                    break;
                }
                _ => {}
            }
        }
    }

    Ok(())
}