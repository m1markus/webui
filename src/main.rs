use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::process::Command;
use std::thread;
use std::time::Duration;

// https://actix.rs/docs/getting-started/

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut show_ui = false;

    for arg_key in std::env::args() {
        if arg_key == "--ui" {
            show_ui = true;
        }
    }

    println!("starting webapp...");

    if show_ui {
        thread::spawn(|| {
            start_browser_window();
        });
    }

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn start_browser_window() {
    thread::sleep(Duration::from_millis(1000));
    // let ui_url = String::from("http://localhost:8080/");
    let ui_url = String::from("http://localhost:8080/");

    let arg_app_url = format!("--app={}", ui_url);
    Command::new("/opt/google/chrome/chrome")
        .arg(arg_app_url)
        .output()
        .expect("failed to execute process");
}
