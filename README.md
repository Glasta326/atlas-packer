# Atlas-packer

This program collects all images in a target folder and subfolders and compiles them into a texture atlas image with a sibling metadata .JSON file
The output of this program is designed for a seperate project of mine to use in its execution


## Installation

This will come shipped as an executable file within the main project this code is for, but you can build and use on its own:

```bash
git clone https://github.com/Glasta326/atlas-packer.git
cd atlas-packer
cargo build --release
```

Note: Requires you have rust and associated cargo packages installed on your system. See https://rust-lang.org/learn/get-started/ for details.

## Usage

Prepare a folder of image files you want to use, and copy the executable into that folder, run it and wait for the files:
- output.png
- output.json
to appear.

Alternatively, you can do the same setup and run it via a terminal to see progress information

### Extra usage info / CLI options

```
Options:
    [-v | --version]: Display the current application version
    [-h | --help]: Display this help text
    [-t | --target]: Specify a target folder to use. Default behaviour operates in the same folder as the executable is run in
    [-n | --norotate]: Disable rotation of images when being packed into the atlas
    [-p | --padding <value>]: Set the amount of empty space padding between images packed into the atlas

Examples:
    ./atlas-packer -t /home/MyUser/Downloads -n
    ./atlas-packer -p 16
```
