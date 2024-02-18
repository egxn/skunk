use axum::Router;
use axum::routing::get;
use std::{env, fs::File};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use zip::read::ZipArchive;

fn uncompress (path: &str) -> &str {
    let file = File::open(path)
        .expect("Failed to open file");
    let mut archive = ZipArchive::new(file)
        .expect("Failed to read zip");
    
    let stem = Path::new(path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    std::fs::create_dir_all(stem)
        .expect("Failed to create dir");

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = Path::new(stem).join(file.name());
        let mut outfile = File::create(&outpath).unwrap();
        std::io::copy(&mut file, &mut outfile).unwrap();
        print!("File {}: File extracted\n", file.name());
    }

    stem
}

fn get_file_path () -> Result<String, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args()
        .collect();
    if args.len() < 2 {
        println!("Please provide a file path as an argument");
        Err("No file path provided")?;
    }

    let path = &args[1];
    Ok(path.to_string())
}

fn get_file_list(path: &str)  -> String {
    let dir = std::fs::read_dir(path)
        .unwrap();

    let mut files: Vec<String> = Vec::new();

    for entry in dir {
        let entry = entry.unwrap();
        let path = entry.path();
        let path = path.to_str().unwrap();
        files.push(path.to_string());
    }

    serde_json::to_string(&files).unwrap()
}

fn write_index(name: &str) -> io::Result<()> {
    let file_path = Path::new("static").join("index.html");
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);
    let mut new_content = String::new();

    for line in reader.lines() {
        let line = line?;
        let replaced_line = line.replace("{{FILE}}", name);
        new_content.push_str(&replaced_line);
        new_content.push('\n');
    }

    let mut file = File::create(&file_path)?;
    file.write_all(new_content.as_bytes())?;

    Ok(())
}

fn copy_file () -> io::Result<()> {
    let source_file_path = "index_template.html";
    let destination_file_path = Path::new("static")
        .join("index.html");

    let source_file = File::open(&source_file_path)?;
    let mut reader = BufReader::new(source_file);

    let destination_file = File::create(&destination_file_path)?;
    let mut writer = BufWriter::new(destination_file);
    let mut buffer = Vec::new();
    
    reader.read_to_end(&mut buffer)?;
    writer.write_all(&buffer)?;

    Ok(())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let path = get_file_path();
    match path {
        Ok(path) => {
            let stem = uncompress(&path);

            copy_file().unwrap();
            write_index(stem).unwrap();

            let path = format!("/{}", stem);
            let meta = format!("/{}-meta", stem);
            let app = Router::new()
                .route(meta.as_str(), get(get_file_list(stem)))
                .nest(path.as_str(), axum_static::static_router(stem))
                .nest("/home", axum_static::static_router("static"));
    
            axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
                .serve(app.into_make_service())
                .await
                .unwrap();
        },
        Err(e) => println!("Error: {}", e)
    }
}
