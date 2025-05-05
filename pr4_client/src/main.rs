use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[tokio::main]
async fn main() {
    // Подключаемся к серверу
    let mut stream = TcpStream::connect("127.0.0.1:8090").await.unwrap();
    println!("Connected to server!");

    loop {
        
    }
    // // Отправляем сообщение серверу
    // let msg = b"Hello from client!\n";
    // stream.write_all(msg).await;
    // println!("Message sent!");

    // // Читаем ответ (если сервер что-то отправляет)
    // let mut buf = vec![0; 1024];
    // let n = stream.read(&mut buf).await.unwrap();
    // if n > 0 {
    //     println!("Received from server: {}", String::from_utf8_lossy(&buf[..n]));
    // } else {
    //     println!("No response from server.");
    // }
}
