use std::process::ExitCode;
use r3y_faces_example::app;

#[tokio::main]
async fn main() -> ExitCode {
    match app::run().await {
        Ok(summary) => {
            println!(
                "Processed: {}/{} images, {} faces found",
                summary.processed, summary.total, summary.faces_found);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
