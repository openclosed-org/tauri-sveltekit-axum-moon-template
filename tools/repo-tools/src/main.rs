mod cli;
mod commands;
mod core;
mod support;

fn main() -> anyhow::Result<()> {
    cli::run()
}
