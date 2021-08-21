#[macro_use]
extern crate log;

use actix_files::NamedFile;
use actix_web::cookie::Cookie;
use actix_web::{
    get, web, App, /* post,*/ HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use serde::{Deserialize, Serialize};
use simple_log::LogConfigBuilder;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::fs;
use std::env;

// upload...
// https://medium.com/swlh/build-your-first-rest-api-using-rust-language-and-actix-framework-8f827175b30f

#[get("/*")]
async fn hello(req: HttpRequest) -> impl Responder {
    let mut path = req.path();
    if path == "/" {
        path = "/index.html";
    }

    info!("requested url: {}", path);

    //let static_resource_directory = "/home/mue/work/git/webapp/res";
    let static_resource_directory = "./res";

    let mut curr_dir = env::current_dir().unwrap();
    info!("current directory is: {}", curr_dir.display());
    curr_dir.push("res");
    info!("resource directory is: {}", curr_dir.display());

    // path is allready prefixed with: '/'
    let full_path = format!("{}{}", static_resource_directory, path);

    info!("path in filesystem: {}", full_path);

    match req.headers().get("host") {
        None => (),
        Some(req_hdr_host) => match req_hdr_host.to_str() {
            Err(_) => (),
            Ok(host_header_value) => {
                info!("in request header 'host' is: {}", host_header_value);
            }
        },
    }

    let req_token_cookie: Cookie = Cookie::build("authtokenreq", "deadbeef")
        .domain("example.com")
        .path("/")
        .secure(false)
        .http_only(true)
        .max_age(time::Duration::minutes(24 * 60))
        .finish();

    info!("cookie created {}", req_token_cookie);

    let file_content_body: String;

    // FIXME: make the code async
    match fs::read_to_string(full_path) {
        Err(_) => file_content_body = format!("file not found: {}", path),
        Ok(content_body) => file_content_body = content_body    
    }

    HttpResponse::Ok()
    .cookie(req_token_cookie)
    .body(file_content_body)
}

#[derive(Serialize, Deserialize, Debug)]
struct AppStatus {
    version: String,
}

#[get("/status")]
async fn get_status() -> impl Responder {
    let app_status = AppStatus {
        version: String::from(clap::crate_version!()),
    };
    info!("requested url /status");
    HttpResponse::Ok().json(app_status)
}

async fn manual_is_ready() -> impl Responder {
    info!("requested url /ready");
    HttpResponse::Ok().body("ok")
}

#[get("/pdf")]
async fn pdf() -> Result<NamedFile> {
    info!("requested url /pdf");
    let path = Path::new("/home/mue/Downloads/sample.pdf");
    Ok(NamedFile::open(path)?)
}

#[get("/favicon.ico")]
async fn favicon() -> Result<NamedFile> {
    info!("requested url /favicon.ico");
    let path = Path::new("/home/mue/Downloads/favicon.ico");
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // set defaults
    //
    const DEFAULT_LOGLEVEL: &str = "warn";
    const DEFAULT_BIND_IP: &str = "127.0.0.1";
    const DEFAULT_PORT: &str = "8080";
    let mut show_ui = false;

    let cliarg = build_command_line_args();
    if cliarg.is_present("ui") {
        show_ui = true;
    }

    let loglevel = cliarg.value_of("loglevel").unwrap_or(DEFAULT_LOGLEVEL);
    setup_logging(&loglevel);
    info!("starting application...");

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings")).unwrap()
        .merge(config::Environment::with_prefix("APP")).unwrap();

    match settings.get_str("loglevel") {
        Ok(loglevel) => info!("loglevel from config file is: {}", loglevel),
        Err(e) => info!("loglevel not found in config file: {}", e),
    }

    let port = cliarg.value_of("port").unwrap_or(DEFAULT_PORT);
    let bind_ip = cliarg.value_of("ip").unwrap_or(DEFAULT_BIND_IP);

    let bind_ip_port = format!("{}:{}", bind_ip, port);
    //let web_url = format!("http://{}", bind_ip_port);
    let web_url = format!("http://example.com:{}", port);

    if show_ui {
        thread::spawn(|| {
            start_browser_window(web_url);
        });
    }

    info!("binding on ip {}", bind_ip_port);

    // start web server
    // https://actix.rs/docs/getting-started/
    //
    HttpServer::new(|| {
        App::new()
            .service(get_status)
            .service(pdf)
            .service(favicon)
            .service(actix_files::Files::new("/static", "/tmp").show_files_listing())
            .service(hello)
            .route("/ready", web::get().to(manual_is_ready))
    })
    .bind(bind_ip_port)?
    .run()
    .await
}

fn build_command_line_args() -> clap::ArgMatches<'static> {
    //
    // https://docs.rs/clap/2.33.3/clap/
    //
    return clap::App::new("webapp")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::with_name("ui")
                .help("Start the WebUI")
                .long("ui"),
        )
        .arg(
            clap::Arg::with_name("ip")
                .help("Bind IP address e.g. 0.0.0.0 for all")
                .value_name("IP_ADDR")
                .long("ip"),
        )
        .arg(
            clap::Arg::with_name("loglevel")
                .help("Levels can be debug|info|warn|error")
                .value_name("LEVEL")
                .long("loglevel"),
        )
        .arg(clap::Arg::with_name("port").value_name("PORT").long("port"))
        .get_matches();
}

fn setup_logging(level: &str) {
    //
    // https://docs.rs/simple-log/1.0.2/simple_log/
    //
    let config = LogConfigBuilder::builder()
        //   .path("./log/builder_log.log")
        //   .size(1 * 100)
        //   .roll_count(10)
        //   .output_file()
        .level(level)
        .output_console()
        .build();

    simple_log::new(config).unwrap();
    //simple_log::quick().unwrap();

    // test some levels
    //
    //error!("test quick error");
    //warn!("test quick warn");
    //info!("test quick info");
    //debug!("test quick debug");
}

fn start_browser_window(ui_url: String) {
    wait_til_web_server_is_ready(&ui_url);

    if cfg!(target_os = "macos") {
        start_chrome_on_osx(ui_url);
        //warn!("browser start not implemented");
    } else if cfg!(unix) {
        start_chrome("chrome".to_string(), ui_url);
    } else if cfg!(target_os = "windows") {
        start_chrome("chrome.exe".to_string(), ui_url);
    } else {
        warn!("browser start not implemented (default)");
    }
}

// osx: open -a "Google Chrome" "$1"

fn start_chrome_on_osx(ui_url: String) {
    Command::new("open")
        .arg("-a")
        .arg("Google Chrome")
        .arg(ui_url)
        // FIXME: size the windows as needed
        //.arg(arg_browser_size)
        .output()
        .expect("failed to execute process");
}

fn start_chrome(bin_name: String, ui_url: String) {
    let arg_app_url = format!("--app={}", ui_url);
    Command::new(bin_name)
        .arg(arg_app_url)
        // FIXME: size the windows as needed
        //.arg(arg_browser_size)
        .output()
        .expect("failed to execute process");
}

fn wait_til_web_server_is_ready(ui_url: &String) {
    let ready_url = format!("{}/status", ui_url);

    info!("wait for webapp to be ready...{}", &ready_url);

    loop {
        // https://github.com/algesten/ureq
        //
        match ureq::get(&ready_url).call() {
            Ok(response) => {
                let http_version = response.http_version();
                info!("http GET url={} server response success http_version={}, let's launch the web-ui",
                &ready_url, http_version);
                break;
            }
            Err(ureq::Error::Status(code, response)) => {
                let http_version = response.http_version();
                error!(
                    "http GET url={} server response with error http_code={} http_version={}",
                    &ready_url, code, http_version
                );
            }
            Err(_) => {
                error!("http GET general error: ");
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

/*
// use to debug (async is not easy to debug)
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

// ******************* google authenticator *******************

// [dependencies]
// google-authenticator = "0.2.0"
// qrcode-generator = "4.1.0"

use google_authenticator::GoogleAuthenticator;
use qrcode_generator::QrCodeEcc;

fn main() {

    // google authenticator
    // https://github.com/google/google-authenticator/wiki/Key-Uri-Format
    //
    let auth = GoogleAuthenticator::new();
    let secret_seed = auth.create_secret(48);
    println!("generated secret is: {}", secret_seed);
    let code_given_by_user = auth.get_code(&secret_seed, 0).unwrap();
    println!("code is: {}", code_given_by_user);
    let verify_result = auth.verify_code(&secret_seed, &code_given_by_user, 1, 0);
    println!("verify is: {}", verify_result);

    let qr_code_data = google_authenticator_generate_data_string("m1m", &secret_seed);

    let output_qr_file = "/tmp/qr_file_output.png";
    qrcode_generator::to_png_to_file(qr_code_data, QrCodeEcc::Medium, 1024, output_qr_file).unwrap();
    println!("qr file created at: file://{}", output_qr_file);
}

fn google_authenticator_generate_data_string(issuer: &str, secret_seed: &str) -> String {
    // generate something like this
    // otpauth://totp/Example:alice@google.com?secret=JBSWY3DPEHPK3PXP&issuer=Example
    //format!("otpauth://totp/{}?secret={}&issuer={}", issuer, secret_seed, issuer)
    format!("otpauth://totp/{}?secret={}", issuer, secret_seed)
}
*/
