use crate::cli::Cli;
use crate::progress::{ProgressEvent, ProgressSummary};
use crate::scanner::ScanMessage;
use crate::worker::{WorkerPool, WorkerResult};
use crate::{progress, scanner};
use std::path::PathBuf;
use std::sync::Arc;

pub async fn run(args: Cli) -> Result<ProgressSummary, String> {
    validate_and_prepare(&args)?;

    let input_dir: PathBuf = args.input_dir.clone();
    let output_dir: PathBuf = args.output_dir.clone();
    let worker_count = args.worker_count();

    let (scan_tx, scan_rx) = tokio::sync::mpsc::channel::<ScanMessage>(scanner::BATCH_CAPACITY);
    let (job_tx, job_rx) = std::sync::mpsc::channel::<PathBuf>();
    let (results_tx, results_rx) = std::sync::mpsc::channel::<WorkerResult>();

    let job_receiver = Arc::new(std::sync::Mutex::new(job_rx));
    let worker_pool = WorkerPool::start(
        worker_count,
        job_receiver,
        results_tx,
        input_dir.clone(),
        output_dir.clone(),
    );

    let (progress_tx, progress_rx) = tokio::sync::mpsc::channel::<ProgressEvent>(256);
    let scanner_bridge_handle = spawn_scanner_bridge(scan_rx, job_tx, progress_tx.clone());
    let results_bridge_handle = spawn_results_bridge(results_rx, progress_tx.clone());
    let progress_task_handle = tokio::spawn(progress::run(progress_rx));

    let scan_task_handle = tokio::spawn(scanner::scan_recursive(input_dir.clone(), scan_tx));
    let scan_outcome = scan_task_handle
        .await
        .map_err(|error| format!("scan task panicked: {error}"))?;

    for error in &scan_outcome.errors {
        eprintln!("scan warning: {error}");
    }

    let _ = scanner_bridge_handle.await;
    let _ = results_bridge_handle.join();
    worker_pool.join();

    drop(progress_tx);
    progress_task_handle
        .await
        .map_err(|error| format!("progress task panicked: {error}"))
}

fn spawn_scanner_bridge(
    mut scan_rx: tokio::sync::mpsc::Receiver<ScanMessage>,
    job_tx: std::sync::mpsc::Sender<PathBuf>,
    progress_tx: tokio::sync::mpsc::Sender<ProgressEvent>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(message) = scan_rx.recv().await {
            match message {
                ScanMessage::Image(path) => {
                    let _ = job_tx.send(path);
                }
                ScanMessage::Done { total } => {
                    progress_tx
                        .send(ProgressEvent::ScanCompleted { total })
                        .await
                        .ok();
                }
            }
        }
    })
}

fn spawn_results_bridge(
    results_rx: std::sync::mpsc::Receiver<WorkerResult>,
    progress_tx: tokio::sync::mpsc::Sender<ProgressEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        while let Ok(result) = results_rx.recv() {
            if progress_tx
                .blocking_send(ProgressEvent::ImageProcessed(result))
                .is_err()
            {
                break;
            }
        }
    })
}

fn validate_and_prepare(args: &Cli) -> Result<(), String> {
    args.validate()?;

    std::fs::create_dir_all(&args.output_dir).map_err(|error| {
        format!(
            "cannot create output directory '{}': {error}",
            args.output_dir.display()
        )
    })
}
