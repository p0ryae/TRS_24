# TRS_24

![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/trs_24.svg)](https://crates.io/crates/trs_24)
[![Downloads](https://img.shields.io/crates/d/trs_24.svg)](https://crates.io/crates/trs_24)
[![Docs](https://docs.rs/trs_24/badge.svg)](https://docs.rs/trs_24/latest/trs_24/)

An OpenGL-Powered Android Game Engine in Rust (OpenGL ES 2.0+)

## Why TRS_24?

It aims to ease game development on Android platforms, focusing on ensuring compatibility, even for older device architectures.
The engine supports OpenGL ES 2.0 graphics API, allowing compatibility with a broad range of Android devices, even those with older hardware. 
Moreover, The process for bundling `.apk` for Android shipment has been significantly untangled.

## Getting started
Fundamentally, using two files is the ideal approach for using the engine. Primary file `lib.rs` soley used for running the window on Android and building a shared object (*.so).
Secondary file `main.rs` solely used for testing purposes on the host machine. Both of these files need to be in the `src` directory.

For both of these files to co-exist, the following needs to be in `Cargo.toml`:
```toml
[lib]
# Causes the production of a dynamic system library
crate-type = ["cdylib"]

[[bin]]
name = "test"
path = "src/main.rs"
```
Both files will have their own respectable structure:

Structure for `lib.rs`:
   
⚠️ Notice the nessesity of `#![cfg(target_os = "android")]` at the 1st line of the file, and the `#[no_mangle]` attribute before the `android_main` function. They NEED to exist, otherwise you'll run to errors and crashes for compliation and building, respectively.

```rust
#![cfg(target_os = "android")]

use trs_24::{android_logger, overture::*};

#[no_mangle]
pub fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default());

    // Creates an event loop for android platforms only.
    let event_loop = EventLoopBuilder::new().with_android_app(app).build();

    // The rest of your code here...
    // 99% of the time, this is the place for the content of the 
    // main function in main.rs, excluding the event_loop definition 
}
```

Structure for `main.rs`:
```rust
use trs_24::overture::*;

pub fn main() {
    // Creates an event loop for non-android platforms only.
    let event_loop = EventLoopBuilder::new().build();

    // The rest of your code here...
}
```
ℹ️ To try and view a fully complete example, clone this repo, and go to the `example` directory.
