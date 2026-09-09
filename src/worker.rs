use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use crate::detector::build_detector;
use crate::image_processor::{process_image, ProcessedImage};

pub struct WorkerResult {
    pub input_path: PathBuf,
    pub image: ProcessedImage
}

pub struct WorkerPool {
    workers: Vec<std::thread::JoinHandle<()>>,
}

impl WorkerPool {
    pub fn start(
        worker_count: usize,
        job_receiver: Arc<std::sync::Mutex<mpsc::Receiver<PathBuf>>>,
        result_sender: mpsc::Sender<WorkerResult>,
        input_root: PathBuf,
        output_dir: PathBuf,
    ) -> WorkerPool {
        let workers = (0..worker_count)
            .map(|_| {
                spawn_worker(
                    Arc::clone(&job_receiver),
                    result_sender.clone(),
                    input_root.clone(),
                    output_dir.clone(),
                )
            }).collect();

        WorkerPool{workers}
    }

    pub fn join(self) {
        for worker in self.workers {
            if let Err(error) = worker.join() {
                eprintln!("worker panicked: {error:?}");
            }
        }
    }
}

fn spawn_worker(
    job_receiver: Arc<std::sync::Mutex<mpsc::Receiver<PathBuf>>>,
    result_sender: mpsc::Sender<WorkerResult>,
    input_root: PathBuf,
    output_dir: PathBuf,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move ||{
        let mut detector = build_detector().expect("Failed to build detector");

        loop {
            let path = match job_receiver.lock().unwrap().recv() {
                Ok(path) => path,
                // if the producer closes the channel, exit the loop
                Err(_) => break,
            };

            match process_image(&path, &mut *detector, &input_root, &output_dir) {
                Ok(result) => {
                    let worker_result = WorkerResult{
                        input_path: path,
                        image: result
                    };
                    if result_sender.send(worker_result).is_err() {
                        // If the receiver has closed the channel, exit the loop
                        break;
                    }
                },
                Err(err) => {
                    eprintln!("Error processing '{}': {err}", path.display());
                },
            }
        }
    })
}
