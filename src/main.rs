use clap::Parser;
#[allow(dead_code)]
#[derive(Parser, Debug)]
#[command(name = "gtw")]
#[command(about = "Wikipedia TUI client", long_about = None)]
struct Args {
    #[arg(long)]
    offline: bool,
    #[arg(long)]
    no_animations: bool,
    #[arg(long)]
    theme: Option<String>,
    #[arg(long)]
    log_file: Option<String>,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    tracing::info!(
        "gtw starting offline={} no_animations={}",
        args.offline,
        args.no_animations
    );

    app::run().await.map_err(|e| miette::miette!(e))?;
    Ok(())
}

mod api;
mod app;
mod cache;
mod core;
mod media;
mod render;
