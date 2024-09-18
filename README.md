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

## Support

BeamMM Only runs on Windows, as BeamNG.drive is only officially available on Windows. While BeamNG.drive can run on Linux fairly well with Proton, its data directory is less accessible and therefore difficult to manage mods with 3rd party software. I have no plans to try to support MacOS at any time. I don't even know if the game works on it.

### Pirated version

BeamMM is not tested, and will never be tested, to see if it functions on a pirated version of the game. You should ***not*** pirate BeamNG.drive. It is a beautiful, fantastic, and impressive game and the devs deserve every purchase they get. If you haven't bought it yet, do so!

### OS support for developing, building, or testing

Due to being built in Rust (Rust is awesome!), BeamMM can be built and unit tested on any platform. In fact, as of now, it has been fully developed and tested on Linux aside from "real life" testing on Windows.

## Installation

Click on releases and download the latest release, which is a simple .exe file. There are currently no additional software needed to run the program. See [Usage](#usage).

### Building from source

To build from source, install rust for your platform. Clone this repo and run `cargo build --release`. The binary will be in `target/{platform}/release`.

Building for release does *not* require nightly rust, despite the `rust-toolchain.toml` file.

### [crates.io](https://crates.io)

BeamMM is now on crates.io! You can install it with `cargo install beammm --locked`.

You can add as a dependency with `cargo add beammm`.

## Usage

BeamMM is a CLI program. Run `beammm.exe -h` for help.

## Contributing

Contributions are greatly appreciated! There are no strict guidelines, just please be respectful and patient.

Make sure to build with rust nightly so you can get accurate coverage reports with llvm-cov. Please write tests for features you add or modify and ensure they pass.

Submit an issue for comments/questions/concerns.

## Technical Details

I want to develop this project in a way that is non-intrusive to the BeamNG.drive devs. Rather than modifying any main game files or moving mods around or directly modding, BeamMM simply keeps its own data separate from BeamNG, and modifies only one BeamNG file, usually located at `%LocalAppData%\BeamNG.drive\{version}\mods\db.json`. This file keeps track of the details of the currently installed mods, including repo mods and other, and their status. BeamMM *only* changes the active status of the mods and that's it.

When BeamNG is launched, it reads the file and loads mods accordingly. During my testing it appears that it only really reads once and only writes when there are changes. This means that any BeamMM changes will not be reflected until the game is restarted. It also means that if you make an in-game change to mods, it will overwrite the `db.json` file.

### Installing/updating/removing mods

There are some difficulties with installing mods. Installing local files will be simple, as BeamMM can just copy the files to the mods directory. However, installing from the official repo is more difficult. There is no easy-to-spot API for retrieving mod data. Attempts to capture in-game repo traffic failed using multiple different techniques. Attempting to dissect web traffic on the repo web site did not succeed in finding an api either, as the site appears to be server-side rendered. This leaves the only obvious solution left being scraping the site. I fully plan to continue my non-intrusive approach so I cause as little of a headache to the devs as possible, so I am determined to find a way to implement downloading and installing mods using BeamMM and not relying on manual downloads or the in-game repo, without abusing their servers. Some possiblities I am considering:

* Directly scraping the site on each user's BeamMM when they go to find and install mods.
* Integrating a browser engine or something (which is likely way more complicated than it is worth) in the front end so they essentially install directly from the site.
* Caching mod pages and data in a GitHub repo or somewhere else that users can access. (I'd need to look into possible copyright issues??)

If you have insights, please discuss with me! These features will remain unimplemented until I commit to a method of implementation.

A possible issue with scraping is low reliability, especially if there is a change in the web page structure. Another is if the devs do not want their site scraped. However, looking at `https://beamng.com/robots.txt`, it only disallows the `/library` endpoint for all User-agents and nothing else is forbidden.

If you are a dev or have concerns about this software interfering with servers or causing any other issues, please reach out to me. I'm trying to avoid causing any problems at all.

