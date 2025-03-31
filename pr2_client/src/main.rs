use std::fs;
use std::path::PathBuf;
use request::multipart::Form;
use request::Client;
use std::io::Read;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "file-uploader")]
#[command(about = "Upload multiple files to server")]
struct Args {
    #[arg(help = "Paths to files to upload", required = true, min_values = 1)]
    files: Vec<PathBuf>,
}

async fn upload_files(files: Vec<PathBuf>) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let form = files
        .into_iter()
        .enumerate()
        .fold(Form::new(), |mut form, (index, path)| {
            if path.exists() && path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.to_str().unwrap().to_lowercase() == "txt" {
                        let mut file = fs::File::open(&path)?;
                        let mut content = String::new();
                        file.read_to_string(&mut content)?;
                        
                        return form.part(format!("files[{}]", index), 
                            reqwest::multipart::Part::text(content)
                                .file_name(path.file_name().unwrap().to_str().unwrap()));
                    }
                }
            }
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid file: {}", path.display())
            )) as Box<dyn std::error::Error>)
        });

    let response = client
        .post("http://127.0.0.1:8000/upload")
        .multipart(form?)
        .send()
        .await?;

    Ok(response.text().await?)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    match upload_files(args.files).await {
        Ok(response) => println!("Server response:\n{}", response),
        Err(e) => println!("Error: {}", e)
    }
}