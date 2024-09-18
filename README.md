# BeamMM - BeamNG Mod Manager

BeamMM is a WIP mod manager for BeamNG.Drive written in Rust. This repo itself is the backend library crate and CLI implementation. A GUI implementation is planned for the future, but not yet started.

## Features

- [ ] Download and install mods
- [ ] Install mods from local files
- [ ] Update mods
- [ ] Remove mods
- [ ] Find mods and mod data
- [x] List installed mods
- [x] Presets
  - [x] Create/delete presets
  - [x] List presets
  - [x] Enable/disable presets
  - [x] See preset mods and preset status
  - [ ] Rename presets
  - [x] Add/remove mods from presets
  - [x] Multiple active presets
- [ ] GUI implementation (BeamMM.gui)
- [x] Enabling/disabling mods in bulk or individually
- [ ] Mod metadata (author, description, etc.)
- [x] Custom game data directory

## Why does BeamNG need an external mod manager? Why not just use the built-in in-game manager?

BeamNG.Drive's in-game mod manager is good for a basic manager; however, it is lacking nice features such as presets (similar to profiles, modpackes, etc.) and has terrible performance (like all BeamNG.drive UI, let's be real) even on high-end hardware. This open-source project aims to implement more features and be far more performant. Additionally, being open source, anyone will be able to implement their desired features or contribute in some other way. Pull requests are always welcome.

## OS Support

BeamMM Only runs on Windows, as BeamNG.drive is only officially available on Windows. While BeamNG.drive can run on Linux fairly well with Proton, its data directory is less accessible and therefore difficult to manage mods with 3rd party software. I have no plans to try to support MacOS at any time. I don't even know if the game works on it.

### OS support for developing, building, or testing

Due to being built in Rust (Rust is awesome!), BeamMM can be built and unit tested on any platform. In fact, as of now, it has been fully developed and tested on Linux aside from "real life" testing on Windows.

## Installation

Click on releases and download the latest release, which is a simple .exe file. There are currently no additional software needed to run the program. See [Usage](#usage).

### Building from source

To build from source, install rust for your platform. Clone this repo and run `cargo build --release`. The binary will be in `target/{platform}/release`.

Building for release does *not* require nightly rust, despite the `rust-toolchain.toml` file.

## Usage

BeamMM is a CLI program. Run `beammm.exe -h` for help.

## Contributing

Contributions are greatly appreciated! There are no strict guidelines, just please be respectful and patient.

Make sure to build with rust nightly so you can get accurate coverage reports with llvm-cov. Please write tests for features you add or modify and ensure they pass.

Submit an issue for comments/questions/concerns.

