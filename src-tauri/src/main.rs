// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--smoke-test") {
        match media_tambola_lib::smoke::run_smoke_test() {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("[SMOKE-TEST] ERROR: {}", e);
                std::process::exit(1);
            }
        }
    }

    media_tambola_lib::run();
}
