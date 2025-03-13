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
    upload: TempFile<'f>,
}

impl Upload<'_> {
    // Бог меня простил
    async fn filestats(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut buff = String::new();
        self.upload
            .open()
            .await
            .unwrap()
            .read_to_string(&mut buff)
            .await?;
        let filestats = format!(
            "Имя файла: {} \nСтрок: {}, Слов: {}, Символов: {}",
            self.upload.name().unwrap(),
            buff.lines().count().to_string(),
            buff.split_whitespace().count().to_string(),
            buff.chars().count().to_string()
        );
        Ok(filestats)
    }
}

#[post("/upload", data = "<file>")]
async fn upload(mut file: Form<Upload<'_>>) -> (Status, String) {
    if file.upload.content_type().unwrap().is_text() {
        let upload_dir = Path::new("./downloaded");
        println!("file = {:?}", file.upload);
        let stats = file.filestats().await.unwrap();
        let file_id = Uuid::new_v4()
            .hyphenated()
            .encode_lower(&mut Uuid::encode_buffer())
            .to_owned();
        let file_dist = upload_dir.join(&file_id);

        // println!("destination = {}", &file_dist.display());
        // println!("length = {} bytes", file.upload.len());

        let _ = file.upload.move_copy_to(&file_dist).await;

        (Status::Ok, stats)
    } else {
        (Status::NotAcceptable, "Неверный тип файла".to_owned())
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![upload])
}
