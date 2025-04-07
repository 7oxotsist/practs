use std::{ffi::OsString, fs::File, io::Read, ptr::null};

use clap::{builder::OsStr, Arg, Command, Error};


struct Files {
    name: OsString,
    context: String
}

impl Files {
    fn new(name: OsString, context: String) -> Self {
        Self { name: name, context: context }
    }

    async fn file_analyse(&self) -> String {
        let file_stats = format!(
            "Имя файла: {}\nСтрок: {}\nСлов: {}\nСимволов: {}\n",
            self.name.to_str().unwrap(),
            self.context.lines().count(),
            self.context.split_whitespace().count(),
            self.context.chars().count()
        );
        file_stats
    }
}

async fn file_handler(dists: Vec<String>) -> Vec<Files> {
    let mut vec: Vec<Files> = vec![];
    for dist in dists.iter() {
        let mut buf: String = "".to_string();
        let path = std::path::Path::new(dist);
        let _ = File::open(path).expect("Неверный путь").read_to_string(&mut buf);
        vec.push(Files{name: path.file_name().unwrap().to_owned(), context: buf});
    }
    vec
    /* `Vec<File>` value */
}

#[tokio::main]
async fn main() {

    // парсим аргументы -> ищем файлы -> тащим строки и пихаем в вектор объекта
    // как найти файл? проверка на абсолют пас -> в текущей папке -> при отказе скан 
    // String в куче -> можно забить и хранить в объекте
    // 
    let matches = Command::new("FileProcessor")
        .arg(
            Arg::new("files")
                .short('f')
                .long("files")
                .value_name("FILE")
                .num_args(1..)
                .required(true)
        )
        .get_matches();

    let files = file_handler(matches.get_many::<String>("files")
        .expect("At least one file required")
        .cloned()
        .collect()).await;
    
    for (i, file) in files.iter().enumerate() {
        println!("{}", format!("Файл №{}", i+1));
        println!("{}", file.file_analyse().await);
    }
}
