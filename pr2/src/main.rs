
use std::{ffi::OsString, path::Path, sync::Arc};

use clap::{Arg, Command};
use tokio::fs;

struct Files {
    name: OsString,
    context: String,
}

impl Files {
    async fn file_analyse(&self) -> (String, usize, usize, usize) {
        (
            self.name.to_str().unwrap().to_string(),
            self.context.lines().count(),
            self.context.split_whitespace().count(),
            self.context.chars().count()
        )
    }
}

async fn file_handler(dists: Vec<String>) -> Vec<Files> {
    let mut handles = vec![];
    
    for dist in dists {
        handles.push(tokio::spawn(async move {
            let path = Path::new(&dist);
            let context = fs::read_to_string(path).await.expect("Неверный путь");
            Files {
                name: path.file_name().unwrap().to_owned(),
                context,
            }
        }));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

#[tokio::main]
async fn main() {
    let matches = Command::new("FileProcessor")
        .arg(
            Arg::new("files")
                .short('f')
                .long("files")
                .value_name("FILE")
                .num_args(1..)
                .required(true),
        )
        .get_matches();

    let files = file_handler(
        matches
            .get_many::<String>("files")
            .expect("At least one file required")
            .cloned()
            .collect(),
    )
    .await;

    let mut analysis_handles = vec![];
    for file in files {
        analysis_handles.push(tokio::spawn(async move {
            (file.name.clone(),
             file.file_analyse().await)
        }));
    }
    let mut sum = [0,0];
    for (i, handle) in analysis_handles.into_iter().enumerate() {
        let (name, stats) = handle.await.unwrap();
        println!("Файл №{}: {}", i + 1, name.to_str().unwrap());
        println!("Имя файла: {}\nСтрок: {}\nСлов: {}\nСимволов: {}\n", stats.0, stats.1, stats.2, stats.3);
        sum[0] += stats.2;
        sum[1] += stats.3;
        println!("Итог: {} слов, {} символов", sum[0], sum[1])
    }
}
