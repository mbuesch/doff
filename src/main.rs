use anyhow::{self as ah, Context as _};
use clap::Parser;
use dioxus::desktop::{Config, WindowBuilder, tao};
use image::GenericImageView as _;
use std::path::PathBuf;

mod diff;
mod io;
mod types;
mod ui;

const ICON_PNG: &[u8] = include_bytes!("../assets/logo-128x128.png");

#[derive(Debug, Parser)]
#[command(about = "Döff - Diff Viewer")]
struct Args {
    /// File to open on the left hand side.
    left: Option<PathBuf>,

    /// File to open on the right hand side.
    right: Option<PathBuf>,
}

fn load_window_icon() -> Option<tao::window::Icon> {
    let img = image::load_from_memory(ICON_PNG)
        .map_err(|e| log::warn!("Failed to load window icon: {e}"))
        .ok()?;
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    tao::window::Icon::from_rgba(rgba, width, height)
        .map_err(|e| log::warn!("Failed to create window icon: {e}"))
        .ok()
}

async fn async_main(args: Args) -> ah::Result<()> {
    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Döff")
                .with_always_on_top(false)
                .with_inner_size(tao::dpi::LogicalSize::new(1400, 900))
                .with_window_icon(load_window_icon()),
        )
        .with_menu(None);

    let initial_files = types::InitialFiles {
        left: args.left,
        right: args.right,
    };
    tokio::task::unconstrained(async move {
        dioxus::LaunchBuilder::desktop()
            .with_cfg(config)
            .with_context(initial_files)
            .launch(ui::App);
    })
    .await;

    Ok(())
}

fn main() -> ah::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse command-line arguments.
    let args = Args::parse();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .context("Failed to build Tokio runtime")?
        .block_on(async_main(args))
        .context("Tokio runtime init error")
}
