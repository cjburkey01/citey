extern crate gl_generator;

use gl_generator::{Api, DebugStructGenerator, Fallbacks, Profile, Registry, StructGenerator};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    // GLFW
    println!("cargo:rustc-link-search=C:\\Program Files (x86)\\GLFW\\lib");

    // OpenGL
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("gl_bindings.rs")).unwrap();

    // We want OpenGL 3.3 Core
    let registry = Registry::new(Api::Gl, (3, 3), Profile::Core, Fallbacks::All, []);

    if env::var("CARGO_FEATURE_DEBUG").is_ok() {
        registry
            .write_bindings(DebugStructGenerator, &mut file)
            .unwrap();
    } else {
        registry.write_bindings(StructGenerator, &mut file).unwrap();
    }
}
