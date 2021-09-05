# Playground rust application with webUI
Developed under linux.

## Prerequisite
- Installed rust programming environment +1.51

## Build

```
cargo build
```

## Run (everything after -- are program arguments)
```
cargo run -- --ui
cargo run -- --ui --loglevel info --ip 0.0.0.0
```
It wil run ./target/debug/webapp

## JukeBox plays with
```
http://localhost:8080/jukebox.html
http://dellxps13-1:8080/jukebox.html
```

## Lern Rust
[https://learning-rust.github.io/](https://learning-rust.github.io/)

## Run python web server

To test a .html file run:
```
python -m http.server 8080
```
