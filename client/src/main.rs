use axum::Router;
use axum::routing::get;
use std::{env, fs::File};
use std::path::Path;
use zip::read::ZipArchive;

fn uncompress (path: &str) -> &str {
    let file = File::open(path)
        .expect("Failed to open file");
    let mut archive = ZipArchive::new(file)
        .expect("Failed to read zip");
    
    let stem = Path::new(path).file_stem().unwrap().to_str().unwrap();
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

async fn root() -> &'static str {
    "Skunk!"
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

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let path = get_file_path();
    match path {
        Ok(path) => {
            let stem = uncompress(&path);
            let path = format!("/{}", stem);
            let meta = format!("/{}-meta", stem);
            let app = Router::new()
                .route("/", get(root))
                .route(meta.as_str(), get(get_file_list(stem)))
                .nest(path.as_str(), axum_static::static_router(stem));

            axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
                .serve(app.into_make_service())
                .await
                .unwrap();
        },
        Err(e) => println!("Error: {}", e)
    }
}