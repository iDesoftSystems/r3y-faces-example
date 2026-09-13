use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use rustface::{Detector, ImageData};
use std::path::{Path, PathBuf};

/// Detected face `(x, y, width, height)` in pixels.
pub type FaceRect = (i32, i32, u32, u32);

#[derive(Debug)]
pub struct ProcessedImage {
    pub output_path: PathBuf,
    pub faces: Vec<FaceRect>,
}

/// load the image, detect faces, and annotate the image with bounding boxes.
pub fn process_image(
    path: &Path,
    detector: &mut dyn Detector,
    input_root: &Path,
    output_root: &Path,
) -> Result<ProcessedImage, String> {
    let image = load_image(path)?;
    let faces = detect_faces(&image, detector);
    let output_path = build_output_path(path, input_root, output_root)?;

    save_annotated_image(&image, &faces, &output_path)?;

    Ok(ProcessedImage { output_path, faces })
}

fn load_image(path: &Path) -> Result<image::DynamicImage, String> {
    image::open(path).map_err(|err| err.to_string())
}

fn detect_faces(image: &image::DynamicImage, detector: &mut dyn Detector) -> Vec<FaceRect> {
    // to detect faces in the image, use a grayscale version of the image.
    let gray = image.to_luma8();
    let image_data = ImageData::new(gray.as_raw(), gray.width(), gray.height());

    detector
        .detect(&image_data)
        .iter()
        .map(|face| {
            let bbox = face.bbox();
            (bbox.x(), bbox.y(), bbox.width(), bbox.height())
        })
        .collect()
}

fn build_output_path(
    path: &Path,
    input_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, String> {
    let relative = path.strip_prefix(input_root).map_err(|_| {
        format!(
            "'{}' is not under '{}'",
            path.display(),
            input_root.display()
        )
    })?;

    Ok(output_root.join(relative))
}

fn save_annotated_image(
    image: &image::DynamicImage,
    faces: &[FaceRect],
    output_path: &Path,
) -> Result<(), String> {
    let mut canvas = image.to_rgb8();

    for &(x, y, width, height) in faces {
        draw_hollow_rect_mut(
            &mut canvas,
            Rect::at(x, y).of_size(width, height),
            image::Rgb([0, 255, 0]),
        );
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create dir '{}': {e}", parent.display()))?;
    }

    canvas
        .save(output_path)
        .map_err(|e| format!("cannot save '{}': {e}", output_path.display()))
}
