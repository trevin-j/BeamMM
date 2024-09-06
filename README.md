# BeamMM - BeamNG Mod Manager

BeamMM is a WIP mod manager for BeamNG.Drive written in Rust. This repo itself is the backend library crate and CLI implementation.

BeamMM is NOT in a working state yet. However, this GitHub repo is public to encourage me to continue working on it.

## Why does BeamNG need an external mod manager? Why not just use the built-in in-game manager?

BeamNG.Drive's in-game mod manager is good for a simple manager; however, it is lacking nice features such as presets (a.k.a. profiles) and has terrible performance (like all in-game UI, let's be real) even on high-end hardware. This open-source project aims to implement more features and be far more performant. Additionally, being open source, anyone will be able to implement their desired features or contribute in some other way. Pull requests are always welcome.

When BeamMM gets to its first workable state, it will already support features such as presets and installing and updating mods. It will remain a CLI-only project until I—or someone else—finds the time to implement a GUI with this crate as the backend, which will be easy as this crate is mostly a library crate and will be easy to use in another Rust project.

A projected date for when this project will be functional is not available.

Submit an issue for comments/questions/concerns.

