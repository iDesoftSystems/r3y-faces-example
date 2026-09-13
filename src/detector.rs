use rustface::{Detector, create_detector_with_model, read_model};
use std::io::Cursor;

/// seeta Face embedded model
const MODEL_BYTES: &[u8] = include_bytes!("../models/seeta_fd_frontal_v1.0.bin");

pub fn build_detector() -> Result<Box<dyn Detector>, String> {
    let model = read_model(Cursor::new(MODEL_BYTES)).map_err(|e| e.to_string())?;

    Ok(create_detector_with_model(model))
}
