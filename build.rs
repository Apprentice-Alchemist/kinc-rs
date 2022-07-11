extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=kinc");
    println!("cargo:rerun-if-changed=kinc.h");

    let mut include_paths = vec!["-I", "Kinc/Sources"];
    #[cfg(unix)]
    include_paths.append(&mut vec!["-I", "Kinc/Backends/System/POSIX/Sources"]);
    #[cfg(target_os = "linux")]
    include_paths.append(&mut vec!["-I", "Kinc/Backends/System/Linux/Sources"]);

    include_paths.append(&mut vec!["-I", "Kinc/Backends/Graphics4/OpenGL/Sources"]);
    include_paths.append(&mut vec!["-I", "Kinc/Backends/Graphics5/G5onG4/Sources"]);

    let bindings = bindgen::Builder::default()
        .header("kinc.h")
        .clang_args(include_paths)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
