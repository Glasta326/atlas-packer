use std::{
    collections::HashMap,
    error::{self, Error},
    fmt,
    ops::Add,
    path::PathBuf,
};

use image::ImageError;
mod atlas;
mod image_extract;

fn main() -> Result<(), ImageError> {
    let mut path = std::env::current_dir()?;
    d;
    //Debug folder
    #[cfg(debug_assertions)]
    {
        path = PathBuf::new();
        path.push("/home/Glasta/Projects/Rust/atlas-packer/testing_images");
        println!("Debug mode!");
    }

    println!("Finding files inside folder: {}\n", path.display());
    let files = image_extract::collect_files(&path)?;

    if files.len() == 0 {
        println!("No files with an extension were found!");
        return Ok(());
    }

    println!("\nFound images:");
    let images = image_extract::load_image_array(files)?;
    let (atlas, json) = atlas::gen_atlas(images)?;

    println!("\nSaving output files...");
    let filename = "output".to_string();
    atlas.save(filename.clone().add(".png"))?;
    std::fs::write(filename.clone().add(".json"), json)?;

    println!("Complete!");

    return Ok(());
}
