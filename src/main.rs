use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_is_ready() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // set defaults
    //
    let mut show_ui = false;

    // handle command line arguments
    // https://docs.rs/clap/2.33.3/clap/
    //
    let matches = clap::App::new("webapp")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::with_name("ui")
                .help("Start the WebUI")
                .long("ui"),
        )
        .arg(clap::Arg::with_name("port").value_name("PORT").long("port"))
        .get_matches();

    if matches.is_present("ui") {
        show_ui = true;
    }

    let port = matches.value_of("port").unwrap_or("8080");
    let bind_ip_port = format!("127.0.0.1:{}", port);
    let web_url = format!("http://{}", bind_ip_port);

    println!("starting webapp...{}", web_url);

    if show_ui {
        thread::spawn(|| {
            start_browser_window(web_url);
        });
    }

    // start web server
    // https://actix.rs/docs/getting-started/
    //
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/ready", web::get().to(manual_is_ready))
    })
    .bind(bind_ip_port)?
    .run()
    .await
}

fn start_browser_window(ui_url: String) {
    wait_til_web_server_is_ready(&ui_url);

    let mut chrome_exe_name = String::from("chrome");
    if cfg!(target_os = "windows") {
        chrome_exe_name.push_str(".exe");
    }

    let arg_app_url = format!("--app={}", ui_url);
    Command::new(chrome_exe_name)
        .arg(arg_app_url)
        // FIXME: size the windows as needed
        //.arg(arg_browser_size)
        .output()
        .expect("failed to execute process");
}

fn wait_til_web_server_is_ready(ui_url: &String) {
    let ready_url = format!("{}/ready", ui_url);
    // force an error
    //let ready_url = format!("{}", "http://xxx.example.com/z/y/z.txt");

    println!("wait for webapp to be ready...{}", &ready_url);

    loop {
        match ureq::get(&ready_url).call() {
            Ok(response) => {
                let http_version = response.http_version();
                println!("http GET url={} server response success http_version={}, let's launch the web-ui",
                &ready_url, http_version);
                break;
            }
            Err(ureq::Error::Status(code, response)) => {
                let http_version = response.http_version();
                println!("http GET url={} server response with error http_code={} http_version={}",
                &ready_url, code, http_version);
            }
            Err(_) => {
                println!("http GET general error: ");
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

/*

// old debug test (async is not easy to debug)
//
fn main() {
    let mynum1 = 10;
    let mynum2 = 20;
    let mynum3 = 30;

    // plain argument style
    //
    for arg_key in std::env::args() {
        if arg_key == "--ui" {
            show_ui = true;
        }
    }
}

*/
