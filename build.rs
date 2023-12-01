use std::env;
use std::fs::File;
use std::path::PathBuf;

use cfg_aliases::cfg_aliases;
use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

fn main() {
    cfg_aliases! {
        android: { target_os = "android" },
        egl_backend: { all(feature = "egl", any(windows, unix), not(apple), not(wasm)) },
    }

    // Generate GLES 2.0 bindings
    let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());
    let mut file = File::create(&dest.join("gl_bindings.rs")).unwrap();
    Registry::new(Api::Gles2, (2, 0), Profile::Core, Fallbacks::All, [])
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
}
