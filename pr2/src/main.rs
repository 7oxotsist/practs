#[macro_use]
extern crate rocket;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use std::path::Path;
use uuid::Uuid;
use std::fs;
#[derive(FromForm)]
struct Upload<'f> {
    upload: TempFile<'f>,
}

// async fn filestats() -> &str {
    
// }

#[post("/upload", data = "<file>")]
async fn upload(mut file: Form<Upload<'_>>) -> (Status, &str) {
    if file.upload.content_type().unwrap().is_text() {
        let upload_dir = Path::new("./downloaded");
        println!("file = {:?}", file.upload);

        let file_id = Uuid::new_v4()
            .hyphenated()
            .encode_lower(&mut Uuid::encode_buffer())
            .to_owned();
        let file_dist = upload_dir.join(&file_id);

        println!("destination = {}", &file_dist.display());
        println!("length = {} bytes", file.upload.len());

        let _ = file.upload.move_copy_to(&file_dist).await;

        (Status::Ok, "СЮДА ВЫВОДИТЬ")
    } else {
        (Status::NotAcceptable, "Неверный тип файла")
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![upload])
}
