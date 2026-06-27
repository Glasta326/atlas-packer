use std::path::PathBuf;

use image::{GenericImageView, ImageError, RgbaImage};

use crate::atlas::{self, Region};

const SUPPORTED_EXTENSIONS: [&'static str; 3] = ["png", "webp", "jpg"];

pub fn collect_files(dir: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
    // Collect all of the files inside of the target folder into a list
    let mut image_paths: Vec<PathBuf> = Vec::new();

    // Recursivley search through directories to aggregate a list of all files within
    for entry in std::fs::read_dir(dir)? {
        let p = entry?.path();
        if p.is_dir() {
            image_paths.append(&mut collect_files(&p)?);
        }
        println!("{}", p.display());
        if p.extension().is_none() {
            continue;
        }
        let ext = p.extension().unwrap().to_str().unwrap();
        // Ensure it's a valid image format
        if !SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
            continue;
        }
        image_paths.push(p);
    }
    println!("Found {} files", image_paths.len());
    return Ok(image_paths);
}

#[derive(Clone)]
pub struct TextureData {
    pub name: String,
    pub texture: RgbaImage,
    pub w: u32,
    pub h: u32,
}

impl TextureData {
    pub fn new(name: String, texture: RgbaImage, padding: u32) -> Self {
        let w = texture.dimensions().0 + padding;
        let h = texture.dimensions().1 + padding;
        return TextureData {
            name,
            texture,
            w,
            h,
        };
    }

    pub fn rotate(&mut self) {
        std::mem::swap(&mut self.w, &mut self.h);
    }
}

pub fn load_image_array(
    image_paths: Vec<PathBuf>,
    padding: u32,
) -> Result<Vec<TextureData>, ImageError> {
    // Pre-known amount of images so we can reserve size beforehand instead of making the vec resize every time we add a new one
    // Most pointless optimisation ever but every cpu cycle counts
    let mut tex_array: Vec<TextureData> = Vec::new();
    tex_array.reserve(image_paths.len());

    for entry in image_paths {
        // Unwrap is safe here because the file is guaranteed to exist.
        println!("{}", entry.display());
        let tex = image::ImageReader::open(&entry)
            .unwrap()
            .decode()?
            .into_rgba8();
        let name = entry.file_prefix().unwrap().to_string_lossy().into_owned();

        // println!(
        //     "{}: [Width: {}, Height: {}, Format: {}]",
        //     name,
        //     tex.dimensions().0,
        //     tex.dimensions().1,
        //     entry.extension().unwrap().display()
        // );

        tex_array.push(TextureData::new(name, tex, padding));
    }

    return Ok(tex_array);
}
