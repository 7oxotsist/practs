#[macro_use]
extern crate rocket;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::tokio::io::AsyncReadExt;
use std::path::Path;
use uuid::Uuid;

#[derive(FromForm)]
struct Upload<'f> {
    upload: Vec<TempFile<'f>>,
}

impl Upload<'_> {
    async fn filestats(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut stats = String::new();
        let mut sum = [0,0];
        for (index, file) in self.upload.iter().enumerate() {
            let mut buff = String::new();
            
            file.open()
                .await
                .unwrap()
                .read_to_string(&mut buff)
                .await?;
            
            let file_stats = format!(
                "\nФайл №{}:\nИмя файла: {}\nСтрок: {}\nСлов: {}\nСимволов: {}\n",
                index + 1,
                file.name().unwrap(),
                buff.lines().count(),
                buff.split_whitespace().count(),
                buff.chars().count()
            );
            sum[0] += buff.split_whitespace().count();
            sum[1] += buff.chars().count();

            stats.push_str(&file_stats);
            
        }
        let filesumm = format!("\nИтог: {} слов, {} символов", sum[0], sum[1]);
        stats.push_str(&filesumm);
        Ok(stats)
    }
}

#[post("/upload", data = "<file>")]
async fn upload(mut file: Form<Upload<'_>>) -> (Status, String) {
    let upload_dir = Path::new("./downloaded");
    for file in file.upload.iter() {
        if !file.content_type().unwrap().is_text() {
            return (Status::NotAcceptable, "Неверный тип файла".to_owned());
        }
    }

    let stats = file.filestats().await.unwrap();
    
    for file in file.upload.iter_mut() {
        let file_id = Uuid::new_v4()
            .hyphenated()
            .encode_lower(&mut Uuid::encode_buffer())
            .to_owned();
        let file_dist = upload_dir.join(&file_id);
        
        let _ = file.move_copy_to(&file_dist).await;
        let _ = std::fs::remove_file(file.path().unwrap());
    }
    
    (Status::Ok, stats)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![upload])
}
