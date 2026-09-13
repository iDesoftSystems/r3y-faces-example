use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Input directory to scan
    #[arg(short, long, value_name = "DIR")]
    pub input_dir: PathBuf,

    /// Output directory to write processed files
    #[arg(short, long, value_name = "DIR")]
    pub output_dir: PathBuf,

    /// Number of threads to use
    #[arg(long, value_name = "N")]
    pub threads: Option<usize>,
}

impl Cli {
    pub fn worker_count(&self) -> usize {
        self.threads
            .or_else(default_worker_count)
            .expect("worker cunt must be at least 1")
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.input_dir.is_dir() {
            return Err(format!(
                "input directory '{}' does not exist or is not a directory",
                self.input_dir.display()
            ));
        }

        Ok(())
    }
}

/// Returns the default number of worker threads to use.
fn default_worker_count() -> Option<usize> {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .ok()
        .filter(|&n| n >= 1)
}
