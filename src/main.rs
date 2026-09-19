use std::{ops::Add, path::PathBuf};
mod atlas;
mod image_extract;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args_os().any(|a| a == "-h" || a == "--help") {
        println!(
            "
        Usage: ./atlas-packer (OPTIONS)
        
        Options:
            [-v | --version]: Display the current application version
            [-h | --help]: Display this help text
            [-i | --input]: Specify a target folder to collect images from. Default behaviour collects from the same folder as the executable is run in
            [-o | --output]: Specify a target folder to output the resulting atlas files into
            [-nr | --norotate]: Disable rotation of images when being packed into the atlas
            [-p | --padding <value>]: Set the amount of empty space padding between images packed into the atlas
            [-n | --name <value>]: Set the output file names

        Examples:
            ./atlas-packer -i /home/MyUser/Downloads -n all_my_downloaded_images
            ./atlas-packer -p 16
        "
        );
        return Ok(());
    } else if std::env::args_os().any(|a| a == "-v" || a == "--version") {
        println!("{}: {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut input_path = std::env::current_dir()?;
    let mut output_path = std::env::current_dir()?;
    let mut args = std::env::args_os().skip(1);

    let mut canrotate = true;
    let mut padding = 0;
    let mut file_name = "output".to_string();
    while let Some(arg) = args.next() {
        match arg.to_string_lossy().to_lowercase().as_ref() {
            "-i" | "--input" => {
                let input_folder = args.next().ok_or(format!("{} was used, but no input folder path was provided.\nHint: use -h or --help for info",arg.display()))?;
                input_path = PathBuf::from(input_folder);
            }
            "-o" | "--output" => {
                let output_folder = args.next().ok_or(format!("{} was used, but no output folder path was provided.\nHint: use -h or --help for info",arg.display()))?;
                output_path = PathBuf::from(output_folder);
            }
            "-nr" | "--norotate" => {
                canrotate = false;
                println!("Disabled rotation.");
            }
            "-p" | "--padding" => {
                let v = args.next().ok_or(format!(
                    "{} was used, but no value was provided.\nHint: use -h or --help for info",
                    arg.display()
                ))?;
                padding = v.to_string_lossy().parse()?;
            }
            "-n" | "--name" => {
                let n = args.next().ok_or(format!(
                    "{} was used, but no file name was provided.\nHint: use -h or --help for info",
                    arg.display()
                ))?;
                file_name = n.to_string_lossy().to_owned().to_string();
            }
            _ => {
                return Err(format!("Unknown parameter: '{}'", arg.display()))?;
            }
        }
    }

    // //Debug folder
    // #[cfg(debug_assertions)]
    // {
    //     path = PathBuf::new();
    //     path.push("/home/Glasta/Projects/Rust/atlas-packer/testing_images");
    //     println!("Debug mode!");
    // }

    // TODO: cli arg to enable/disable recursive search
    println!("Finding files inside folder: {}\n", input_path.display());
    let files = image_extract::collect_files(&input_path)?;

    if files.len() == 0 {
        println!("No files with an extension were found!");
        return Ok(());
    }

    println!("\nFound images:");
    let images = image_extract::load_image_array(files, padding)?;
    let (atlas, json) = atlas::gen_atlas(images, canrotate)?;

    println!("\nSaving output files...");
    let mut output_image_fp = output_path.clone();
    output_image_fp.push(file_name.clone().add(".png"));
    atlas.save(output_image_fp)?;

    let mut output_json_fp = output_path.clone();
    output_json_fp.push(file_name.clone().add(".json"));
    std::fs::write(output_json_fp, json)?;

    println!("Complete!");

    return Ok(());
}
