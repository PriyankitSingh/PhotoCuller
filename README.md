# Photo Culler

## Overview
A high-performance, cross-platform desktop application for quickly reviewing and culling paired RAW+JPEG photo files. Photographers who shoot RAW+JPEG need to quickly review photos and decide which formats to keep or delete. This tool provides a keyboard-driven interface to mark files for deletion and bulk delete them. Built with Rust for fun

## How to build?
Go to repository directory and run the following. 
``cargo run --release``

## How to use
A lot of cameras store images of different formats in pairs. For example, the same images will be stored with the same name but with different file extensions. Use this tool to cycle through image pairs and mark different formats for deletion. The deletion shortcuts are as follows:
- Press 2 to delete RAW image. 
- Press 3 to delete JPEG.
- Press 4 to delete both formats.
### Here is an example workflow:
1. Open a folder containing images to sort using ``Ctrl+O``.
2. Cycle through images using the arrow keys
3. For the displayed image, mark RAW file for deletion.
4. Press ``Ctrl+D`` to delete all marked files.
