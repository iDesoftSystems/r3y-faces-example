# r3y-faces-example

This repository contains a final example project that applies the concepts covered in the *Fearless Concurrency* course. After working through the theory and the basic examples of each topic, this multi-threaded face-detection CLI brings them all together: it scans a directory recursively, detects faces in each image in parallel using a worker pool, and writes annotated copies with bounding boxes drawn over the faces.

## Video Lessons

- Italiano: [Guarda la Playlist su YouTube](https://www.youtube.com/playlist?list=PLSLcKcqBWfjJ0fWnUSIKK66U7OapnAXls)
- Español: [Mira la Playlist en YouTube](https://www.youtube.com/playlist?list=PL8aBwUBHv2TgGw9pG_Td3atFeKJQGTXNg)

## Curriculum Overview

- Module 1: Safe Multi-threading - Synchronization mechanisms and secure thread management
- Module 2: Asynchronous Programming - Fundamentals to optimize workflows and understand how async programming comes to the rescue
- Module 3: Tokio Runtime - The industry standard for asynchronous networking and efficient communication with hardware
- Module 4: Advanced Flow Control - Tokio tools to keep threads non-blocking, cooperative multitasking, delays and timeouts

## Getting started from scratch

1. Install [Rust](https://www.rust-lang.org/learn/get-started)
2. Download the *WIDER FACE* dataset from [WIDER FACE](http://shuoyang1213.me/WIDERFACE/) and unzip the images into an input directory (e.g. `data/raw`). The face-detection model is already bundled in this repository, no download needed.
3. Run the pipeline:

   `cargo run --release -- -i data/raw -o data/processed`

Each processed image keeps its relative path under the output directory, and the CLI reports "Processed: X/Y images, N faces found" after finishing.

## Command-line options

- `-i, --input-dir <DIR>` - directory to scan for images (jpg, jpeg, png)
- `-o, --output-dir <DIR>` - directory where annotated images are written
- `--threads <N>` - number of worker threads (default: available cores)