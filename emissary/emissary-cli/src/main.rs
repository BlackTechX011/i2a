use clap::Parser;
use emissary_cli::cli::Arguments;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    emissary_cli::run(arguments).await
}