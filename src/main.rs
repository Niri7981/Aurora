mod cli;
mod config;
mod ollama;
mod session;

fn main() -> Result<(), String> {
    let config = config::load_config(std::env::args().nth(1))?;
    cli::run(&config)
}
