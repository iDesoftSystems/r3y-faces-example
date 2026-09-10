use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc};
use std::sync::atomic::{AtomicUsize, Ordering};

type ScanFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;


pub struct ScanOutcome {
    pub errors: Vec<String>
}

#[derive(Clone)]
pub struct ScanContext {
    sender: tokio::sync::mpsc::Sender<ScanMessage>,
    total: Arc<AtomicUsize>,
    errors: Arc<std::sync::Mutex<Vec<String>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanMessage {
    /// an image ready to be processed
    Image(PathBuf),

    /// all images have been scanned, total is the number of images scanned
    Done {total: usize}
}

pub async fn scan_recursive(
    root: PathBuf,
    tx: tokio::sync::mpsc::Sender<ScanMessage>
) -> ScanOutcome {
    let root_label = root.display().to_string();

    let context = ScanContext {
        sender: tx,
        total: Arc::new(AtomicUsize::new(0)),
        errors: Arc::new(std::sync::Mutex::new(Vec::new())),
    };

    let result = tokio::spawn(scan_dir(root, context.clone())).await;

    if let Err(error) = result {
        context.errors.lock().unwrap().push(
            format!("scan task for '{root_label}' failed: {error}")
        );
    }

    let total = context.total.load(Ordering::SeqCst);
    if total > 0 {
        context.sender.send(ScanMessage::Done {total}).await.ok();
    }

    ScanOutcome{
        errors: Arc::into_inner(context.errors)
            .expect("no other Arc refs alive")
            .into_inner()
            .unwrap()
    }
}


fn scan_dir(dir: PathBuf, context: ScanContext) -> ScanFuture {
    Box::pin(async move {
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(entries) => entries,
            Err(error) => {
                record_error(
                    &context,
                format!("cannot read dir '{}': {error}", dir.display())
                );
                return;
            }
        };

        let mut image_paths = Vec::new();
        let mut sub_dir_tasks = Vec::new();

        loop {
            let entry = match entries.next_entry().await {
                Ok(Some(entry)) => entry,
                Ok(None) => break,
                Err(error) => {
                    record_error(
                        &context,
                        format!("cannot list '{}': {error}", dir.display())
                    );
                    break;
                }
            };

            let path = entry.path();
            let is_directory = match tokio::fs::metadata(&path).await {
                Ok(metadata) => metadata.is_dir(),
                Err(error) => {
                    record_error(
                        &context,
                        format!("cannot inspect '{}': {error}", path.display())
                    );
                    continue
                }
            };

            if is_directory {
                // each subdirectory is scanned in a separate task
                sub_dir_tasks.push(tokio::spawn(scan_dir(path, context.clone())));
            } else if is_image(&path) {
                image_paths.push(path);
            }
        }

        // send all images in the directory to the channel.
        if !send_images(&context, image_paths).await {
            return;
        }

        // wait for all subdirectories to finish scanning
        wait_for_subdirectories(&context, &dir, sub_dir_tasks).await
    })
}

fn record_error(context: &ScanContext, message: String) {
    context.errors.lock().unwrap().push(message);
}

pub fn is_image(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    matches!(ext.to_ascii_lowercase().as_str(), "jpg" | "jpeg" | "png")
}

async fn send_images(context: &ScanContext, image_paths: Vec<PathBuf>) -> bool {
    for path in image_paths {
        match context.sender.send(ScanMessage::Image(path)).await {
            Ok(()) => {
                context.total.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {
                return false;
            }
        }
    }

    true
}

async fn wait_for_subdirectories(
    context: &ScanContext,
    parent: &Path,
    tasks: Vec<tokio::task::JoinHandle<()>>,
) {
    for task in tasks {
        if let Err(error) = task.await {
            record_error(
                context,
                format!("scan task for a subdir of '{}' failed: {error}", parent.display()));
        }
    }
}