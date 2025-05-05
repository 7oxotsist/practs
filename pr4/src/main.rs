use tokio::net::{TcpListener, TcpStream};
use rand;
use rand::{rng, Rng};
use rand::distr::Alphanumeric;

/// Генерирует вектор случайных букв и цифр заданной длины
fn generate_random_code() -> Vec<char> {
    rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(|c| c as char)
        .collect()
    
}

#[tokio::main]
async fn main() -> Result<Ok,>{
    let listener = TcpListener::bind("127.0.0.1:8090").await.unwrap();
    println!("Listening...");
    let code = generate_random_code();
    println!("Code generated.");
    loop {
        // Вектор для хранения активных соединений
    let mut streams: Vec<TcpStream> = Vec::new();

    loop {
        let (conns, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        // Добавляем новое соединение в вектор
        streams.push(conns);

        // Для примера: выводим текущее количество подключений
        println!("Current connections: {}", streams.len());
    }
        
    }
    
}