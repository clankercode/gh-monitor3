#![allow(dead_code)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::too_many_arguments)]

use clap::Parser;
use tracing_subscriber::EnvFilter;

mod animation;
mod app;
mod config;
mod error;
mod github;
mod render;
mod timeline;
mod window;

#[derive(Parser)]
#[command(name = "gh-monitor3", about = "GitHub activity monitor overlay")]
struct Cli {
    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    generate_config: bool,

    #[arg(long)]
    token: Option<String>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    if cli.generate_config {
        let config = config::Config::default();
        config.save(cli.config.as_deref())?;
        println!("Generated default config");
        return Ok(());
    }

    let mut config = config::Config::load(cli.config.as_deref())?;
    if let Some(token) = cli.token {
        config.github_token = Some(token);
    }

    let (app, event_loop) = app::App::new(config);
    app.run(event_loop);

    Ok(())
}
