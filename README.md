# Simple Rust ScratchPad

srsp is a scratchpad for X11 written in Rust.

## Installation

``````
git clone https://github.com/EdenQwQ/srsp
cd srsp
cargo install --path .
``````

## Usage

`srsp -i <WINDOW_ID>` to push a window into the invisible workspace.

`srsp -o <WINDOW_ID>` to pop a window out of the invisible workspace.

You can use `srsp -i focused` to push the focused window, and `srsp -i selected` to select a window with mouse pointer to push. The latter one requires `xdotool` to be installed.

You can use `srsp -o last` to pop out the last window in the scratchpad.
