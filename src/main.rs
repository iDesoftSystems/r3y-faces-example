use clap::Parser;
use r3y_faces_example::app;
use r3y_faces_example::cli::Cli;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Cli::parse();

    match app::run(args).await {
        Ok(summary) => {
            println!(
                "Processed: {}/{} images, {} faces found",
                summary.processed, summary.total, summary.faces_found
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
