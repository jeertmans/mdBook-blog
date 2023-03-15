use chrono::Local;
use clap::{Parser, Subcommand};
use env_logger::Builder;
use log::LevelFilter;
use mdbook::{
    errors::Error,
    preprocess::{CmdPreprocessor, Preprocessor},
};
use semver::{Version, VersionReq};

use mdbook_blog::BlogPreprocessor;

use std::{
    env,
    io::{self, Write},
    process,
};

#[derive(Debug, Subcommand)]
enum Commands {
    /// Check if preprocessor supports renderer.
    Supports {
        /// Renderer.
        renderer: String,
    },
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct App {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Init env. logger
///
/// Adapated from mdBook's owm logger:
/// https://github.com/rust-lang/mdBook/blob/efb671aaf241b7f93597ac70178989a332fe85e0/src/main.rs#LL97-L121C2
fn init_logger() {
    let mut builder = Builder::new();

    builder.format(|formatter, record| {
        writeln!(
            formatter,
            "{} [{}] ({}): {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.args()
        )
    });

    if let Ok(var) = env::var("RUST_LOG") {
        builder.parse_filters(&var);
    } else {
        // if no RUST_LOG provided, default to logging at the Info level
        builder.filter(None, LevelFilter::Info);
        // Filter extraneous html5ever not-implemented messages
        builder.filter(Some("html5ever"), LevelFilter::Error);
    }

    builder.init();
}

fn main() {
    init_logger();
    let app = App::parse();
    let pre = BlogPreprocessor::new();

    match app.command {
        Some(Commands::Supports { renderer }) => handle_supports(pre, &renderer),
        None => {
            if let Err(e) = handle_preprocessing(pre) {
                eprintln!("{}", e);
                process::exit(1);
            }
        },
    }
}

fn handle_preprocessing(pre: impl Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, but we're being \
             called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = &pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: impl Preprocessor, renderer: &str) -> ! {
    let supported = pre.supports_renderer(renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
