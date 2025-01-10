# rust-3d-renderer
simple 3d wireframe renderer in rust

just saw a 3d renderer and wanted to make one myself to learn more about rust and 3d graphics! this is what i came up with - a basic wireframe renderer that can show different 3d shapes.

## what it does
- renders basic 3d shapes (cube, pyramid, octahedron)
- rotates them in 3d space
- uses perspective projection for that 3d look
- lets you switch between shapes with keyboard

## controls
- `1`: show cube
- `2`: show pyramid
- `3`: show octahedron
- `ESC`: quit

## setup
1. make sure you have rust installed ([rust-lang](https://www.rust-lang.org/tools/install))
2. clone this repo
3. to run it:
```bash
cd rust-3d-renderer
cargo run --release
```
4. a window should pop up and you can use the controls to switch between shapes

## notes
- uses minifb for window management
- everything else (3d math, projection, line drawing) is done from scratch
- runs at ~60fps

feel free to mess with the code and learn from it!