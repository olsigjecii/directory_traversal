use actix_files;
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer};
use std::fs;
use std::path::PathBuf; // Renaming for convenience

// Vulnerable function to serve static files
async fn _insecure_serve_static(req: HttpRequest) -> Result<HttpResponse, Error> {
    // 1. Get the requested filename from the URL
    let filename: String = req.match_info().get("filename").unwrap().to_string();

    // 2. UNSAFE: Directly join the user-provided filename to a base path
    let file_path = PathBuf::from("./public/").join(&filename);
    println!("[Attempting to access]: {:?}", file_path);

    // 3. Read the file and return its content
    match fs::read_to_string(&file_path) {
        Ok(content) => Ok(HttpResponse::Ok().body(content)),
        Err(_) => Ok(HttpResponse::NotFound().body("File not found")),
    }
}

// Secure function to serve static files
async fn _serve_static_secure(req: HttpRequest) -> Result<HttpResponse, Error> {
    let base_dir = PathBuf::from("./public");
    let filename: PathBuf = req.match_info().query("filename").parse().unwrap();

    // Combine the base directory with the user's requested file
    let requested_path = base_dir.join(&filename);

    // Canonicalize the path to resolve all `.` and `..` segments.
    // This converts it to an absolute path.
    let canonical_path = match fs::canonicalize(&requested_path) {
        Ok(path) => path,
        Err(_) => return Ok(HttpResponse::NotFound().body("File not found")),
    };

    // Get the absolute path of our safe, base directory
    let canonical_base = fs::canonicalize(&base_dir).unwrap();

    // **THE SECURITY CHECK**
    // Ensure the final, resolved path is still a child of our base directory.
    if !canonical_path.starts_with(canonical_base) {
        println!(
            "[ACCESS DENIED]: Traversal attempt blocked for {:?}",
            &filename
        );
        return Ok(HttpResponse::Forbidden().body("Access Denied"));
    }

    // If the check passes, we can safely serve the file.
    match fs::read_to_string(&canonical_path) {
        Ok(content) => Ok(HttpResponse::Ok().body(content)),
        Err(_) => Ok(HttpResponse::NotFound().body("File not found")),
    }
}

// MANUAL FIX
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     // Setup for demonstration
//     fs::create_dir_all("./public")?;
//     fs::write("./public/index.html", "This is a public file.")?;
//     fs::create_dir_all("./etc")?;
//     fs::write("./etc/passwd", "root:x:0:0:root:/root:/bin/bash")?;

//     println!("🦀 Starting secure server at http://127.0.0.1:8080");
//     HttpServer::new(|| {
//         let static_service =
//             web::scope("/static").route("/{filename:.*}", web::get().to(serve_static_secure));

//         App::new().service(static_service)
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }

// PRODUCTION-READY FIX with `actix-files`
// This version uses `actix-files` to securely serve static files without manual checks.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Setup for demonstration (creating the files to be served)
    std::fs::create_dir_all("./public")?;
    std::fs::write("./public/index.html", "This is a public file.")?;

    println!("🦀 Starting production-ready server at http://127.0.0.1:8080");
    HttpServer::new(|| {
        // `actix-files` creates a service that securely handles static file serving.
        // It prevents directory traversal internally.
        // We mount this service at the "/static" path.
        App::new().service(actix_files::Files::new("/static", "./public"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
