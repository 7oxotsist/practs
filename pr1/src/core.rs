use std::fs;
use std::io;

fn read_file(path: &str) -> Result<String, io::Error> {
    let contents = fs::read_to_string(path)
        .expect("Неверный путь");
    Ok(contents)
}

pub fn hard_search(path: &str, word: &str) {
    let text = read_file(path);
    let count = text.unwrap().split_whitespace()
        .filter(|&w| w == word)
        .count();
    println!("Всего точных совпадений слов в тексте: {count}")
}

pub fn soft_search(path: &str, word: &str) {
    let text = read_file(path);
    let count = text.unwrap().matches(word).count();
    println!("Всего совпадений в тексте: {count}")
}

pub fn word_count(path: &str) {
    let text = read_file(path);
    let mut word_count: usize = 0;
    for _word in text.unwrap().split_whitespace() {
        word_count += 1;
    }
    println!("Найдено слов: {word_count}");
    
}

pub fn print(path: &str) {
    let text = read_file(path).unwrap();
    println!("{text}");
}