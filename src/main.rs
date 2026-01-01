mod cli;
mod config;
mod logger;
mod projects;
mod state;
mod tui;

fn main() {
    eprintln!("[MAIN] Byte starting...");
    logger::info("Byte starting...");

    if let Err(e) = run() {
        logger::error(&format!("Error: {}", e));
        std::process::exit(1);
    }

    logger::info("Byte completed");
    eprintln!("[MAIN] Byte completed");
}

fn run() -> anyhow::Result<()> {
    cli::run()
}
