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
            [-t | --target]: Specify a target folder to use,
            Default behaviour operates in the same folder as the executable is run in
            [-n | --norotate]: Disable rotation of images when being packed into the atlas
            [-p | --padding]: Set the amount of empty space padding between images packed into the atlas

        Examples:
            ./atlas-packer -t /home/MyUser/Downloads/
            ./atlas-packer -n

        Detailed example of console usage:
            User@Computer:~/Desktop$ ls
            1126556862070915214.webp  centrifuge.png  image.png       my-image.png     Q5.png
            atlas-packer              CuteCat.png     Item_Gold.webp  pixelsword.webp  Scenery.png
            pixelsword.png  
            User@Computer:~/Desktop$ ./atlas-packer -n
            > Program output...
            > Program output...
            User@Computer:~/Desktop$ ls
            1126556862070915214.webp  centrifuge.png  image.png       my-image.png  output.png       Q5.png
            atlas-packer              CuteCat.png     Item_Gold.webp  output.json   pixelsword.webp  Scenery.png
        "
        );
        return Ok(());
    } else if std::env::args_os().any(|a| a == "-v" || a == "--version") {
        println!("{}: {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut path = std::env::current_dir()?;
    let mut args = std::env::args_os().skip(1);

    let mut canrotate = true;
    let mut padding = 0;
    while let Some(arg) = args.next() {
        match arg.to_string_lossy().to_lowercase().as_ref() {
            "-t" | "--target" => {
                let target_folder = args.next().ok_or(format!("{} was used, but no folder path was provided.\nHint: use -h or --help for info",arg.display()))?;
                path = PathBuf::from(target_folder);
            }
            "-n" | "--norotate" => {
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
            _ => {
                return Err(format!("Unknown parameter: '{}'", arg.display()))?;
            }
        }
    }

    //Debug folder
    #[cfg(debug_assertions)]
    {
        path = PathBuf::new();
        path.push("testing_images/");
        println!("Debug mode!");
    }

    println!("Finding files inside folder: {}\n", path.display());
    let files = image_extract::collect_files(&path)?;

    if files.len() == 0 {
        println!("No files with an extension were found!");
        return Ok(());
    }

    println!("\nFound images:");
    let images = image_extract::load_image_array(files, padding)?;
    let (atlas, json) = atlas::gen_atlas(images, canrotate)?;

    println!("\nSaving output files...");
    let filename = "output".to_string();
    atlas.save(filename.clone().add(".png"))?;
    std::fs::write(filename.clone().add(".json"), json)?;

    println!("Complete!");

    return Ok(());
}
