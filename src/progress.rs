use crate::worker::WorkerResult;

#[derive(Debug, PartialEq, Eq)]
pub struct ProgressSummary {
    pub processed: usize,
    pub total: usize,
    pub faces_found: usize,
}

pub enum ProgressEvent {
    /// the scan has completed, and we have the total number of images
    ScanCompleted {total: usize},

    /// an image has been processed, and we have the result
    ImageProcessed(WorkerResult)
}

pub async fn run(
    mut events: tokio::sync::mpsc::Receiver<ProgressEvent>
) -> ProgressSummary {
    let mut processed = 0usize;
    let mut faces_found = 0usize;
    let mut total = 0usize;

    while let Some(event) = events.recv().await {
        match event {
            ProgressEvent::ScanCompleted { total: total_scanned } => {
                total = total_scanned;

                println!("scanning {total} images");
            }
            ProgressEvent::ImageProcessed(result) => {
                processed += 1;
                faces_found += result.image.faces.len();

                println!(
                    "{}: {} face(s) -> {}",
                    result.input_path.display(),
                    result.image.faces.len(),
                    result.image.output_path.display()
                );
            }
        }
    }

    ProgressSummary {
        processed,
        total,
        faces_found
    }
}