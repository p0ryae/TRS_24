# TRS_24

![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/trs_24.svg)](https://crates.io/crates/trs_24)
[![Downloads](https://img.shields.io/crates/d/trs_24.svg)](https://crates.io/crates/trs_24)
[![Docs](https://docs.rs/trs_24/badge.svg)](https://docs.rs/trs_24/latest/trs_24/)

An OpenGL-Powered Game Engine in Rust (OpenGL 2.0+) 

## Why TRS_24?

Aims to ease game development on Android & Non-Android platforms, focusing on ensuring compatibility, even for older device architectures.
The engine supports OpenGL 2.0 graphics API. This allows compatibility with a broad range of devices, even those with older hardware. 
Moreover, The process for bundling `.apk` for Android shipment has been significantly untangled.

ℹ️ **Note:** TRS_24 supports platforms other than Android as well.

## Getting Started
Fundamentally, using two files is the ideal approach for using the engine. Primary file `lib.rs` solely used for running the window on Android and building a shared object (*.so).
Secondary file `main.rs` solely used for testing purposes on the host machine, and building for Non-Android platforms. Both of these files need to be in the `src` directory.

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

- Structure for `lib.rs`:
   
⚠️ Notice the necessity of `#![cfg(target_os = "android")]` at the 1st line of the file, and the `#[no_mangle]` attribute before the `android_main` function. They NEED to exist, otherwise you'll run to errors and crashes for compliation and building, respectively.

```rust
#![cfg(target_os = "android")]

use trs_24::overture::*;

#[no_mangle]
pub fn android_main(app: AndroidApp) {
    // Creates an event loop for android platforms only.
    let event_loop = EventLoopBuilder::new().with_android_app(app).build();

    // The rest of your code here...
    // 99% of the time, this is the place for the content of the 
    // main function in main.rs, excluding the event_loop definition 
}
```

- Structure for `main.rs`:
```rust
use trs_24::overture::*;

pub fn main() {
    // Creates an event loop for non-android platforms only.
    let event_loop = EventLoopBuilder::new().build();

    // The rest of your code here...
}
```
ℹ️ To try and view a fully complete example, clone the repository and head to the `example` directory.

## Build & Bundle

### Non-Android

Simply build your app in release mode:
```sh
# Build in release mode
cargo build --release
```

If you are building for a platform that isn't your host platform (i.e. windows executable on linux or etc..), you need to chose a target. Simply head over to [here](https://doc.rust-lang.org/nightly/rustc/platform-support.html#tier-1-with-host-tools) and pick your desired target platform. For the sake of an example, `x86_64-pc-windows-msvc` is used:

```sh
# Add the target platform using rustup
rustup target add x86_64-pc-windows-msvc

# Use the target platform to build
cargo build --release --target=x86_64-pc-windows-msvc
```

### Android

The build process for android can be quite complex, but once understood and configured, it will be reliable. 

1. **Decide on your target rustup architecture(s)**

Here's a list of rustup target architectures for Android:

| Target  | Note |
| ------------- | ------------- |
| `aarch64-linux-android` | ARM64 Android  |
| `arm-linux-androideabi` | ARMv6 Android  |
| `armv7-linux-androideabi` | ARMv7-A Android  |
| `i686-linux-android` | 32-bit x86 Android  |
| `x86_64-linux-android` | 64-bit x86 Android |
| `thumbv7neon-linux-androideabi` | Thumb2-mode ARMv7-A Android with NEON |

2. **Download the right NDK toolset**

You pick this according to your **minimium** supported android version, **AND** target architecture(s).

The version you pick is important, because more recent NDKs drop support for older android versions and architictures. My recommendation is [Android NDK r16b](https://github.com/android/ndk/wiki/Unsupported-Downloads#r16b), since it is the last NDK with support for `armeabi`, `MIPS`, and `MIPS64` architectures. Furthermore, it has support for most common min Android versions. However, I **highly** recommend to up this version or even use the latest NDK, to better suit your development. 

Be sure to check [NDK Revision History](https://developer.android.com/ndk/downloads/revision_history) for more information about the deprecations/orphans that come with the releases.

3. **Set up and Use the NDK toolset for builds**

ℹ️ **Note:** This example uses NDK r16b.

Extract your downloaded NDK toolset, and head to the `build/tools` directory. Create a toolchain  by running the python script `make_standalone_toolchain.py`:
```sh
# Arch flag accepts one of: 'arm', 'arm64', 'mips', 'mips64', 'x86', 'x86_64'
python make_standalone_toolchain.py --arch arm
```

Once the process is over, you will see a new archive file in the directory of the python script. Extract this archive in the tools directory.

Lastly, In the root directory of your rust project, create a directory named `.cargo`, and within that directory, make a file named `config`. Within the `config` file, you'll be linking clang++ and target toolset libraries. Here's an example for `arm-linux-androideabi`:

```toml
[target.arm-linux-androideabi]
linker="/path/to/your/android-ndk-r16b/build/tools/arm/bin/arm-linux-androideabi-clang++"
rustflags=["-L", "/path/to/your/android-ndk-r16b/platforms/android-19/arch-arm/usr/lib"]
```

4. **Build for target architecture (`.so`)**

We're going to build a shared object for an Android architecture, on a host that is non-Android. You picked this target architecture already in **Step 1**. 

We're going to add the target architecture using rustup, and use the target to build (in release mode):
```sh
# Add the target build architecture using rustup
rustup target add arm-linux-androideabi

# Use the target architecture to build
cargo build --release --target=arm-linux-androideabi
```

You then should have produced a shared library (`.so`) file inside of your `target/arm-linux-androideabi/release`. This will be later used for bundling.

If faced with an error while building, then there must be an issue with the NDK Toolset setup. Be sure to read the error for clues. 

5. **Bundling for shipment (`.apk`)**

All left is to bundle. Use this repo's [Android directory](https://github.com/p0ryae/TRS_24/tree/main/android), and copy it to the root of your rust project.

Within the `apk.sh` or `apk.cmd` script, esure you go through the empty options and fill them. Once you're done, run it.


