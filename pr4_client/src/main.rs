use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::io::{self, BufRead, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
   // Подключение к серверу
   let (ws_stream, _) = connect_async("ws://127.0.0.1:8090").await?;
   let (mut write, mut read) = ws_stream.split();

   println!("Connected to server. Type '/ready' to start");
   print!("> ");
   io::stdout().flush()?; 

   let (input_tx, mut input_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

   // Задача для чтения пользовательского ввода (блокирующая, поэтому spawn_blocking)
   let input_handle = tokio::spawn(async move {
       loop {
           let mut input = String::new();
           // Блокирующее чтение stdin
           let _ = tokio::task::spawn_blocking(move || {
               io::stdin().read_line(&mut input).map(|_| input)
           }).await.unwrap().map(|line| {
               let trimmed = line.trim().to_string();
               if !trimmed.is_empty() {
                   let _ = input_tx.send(trimmed);
               }
           });
           print!("> ");
           let _ = io::stdout().flush();
       }
   });

   // Задача для отправки сообщений на сервер
   let send_handle = tokio::spawn(async move {
       while let Some(input) = input_rx.recv().await {
           if let Err(e) = write.send(Message::Text(input.into())).await {
               eprintln!("Send error: {}", e);
               break;
           }
       }
   });

   // Задача для получения сообщений от сервера
   let recv_handle = tokio::spawn(async move {
       while let Some(msg) = read.next().await {
           match msg {
               Ok(Message::Text(text)) => println!("\nServer: {}\n> ", text),
               Ok(Message::Close(_)) => {
                   println!("Connection closed");
                   break;
               }
               Ok(_) => {}
               Err(e) => {
                   eprintln!("Receive error: {}", e);
                   break;
               }
           }
           let _ = io::stdout().flush();
       }
   });

   // Ждем завершения задач
   let _ = tokio::try_join!(input_handle, send_handle, recv_handle);

   Ok(())
}
