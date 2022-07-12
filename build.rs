extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=kinc.h");

    let mut include_paths = vec!["Kinc/Sources"];
    let mut defines = vec!["KINC_NO_MAIN"];

    if cfg!(unix) {
        include_paths.push("Kinc/Backends/System/POSIX/Sources");
    }
    if cfg!(target_os = "linux") {
        include_paths.push("Kinc/Backends/System/Linux/Sources");
        defines.push("KINC_NO_WAYLAND");
    }
    if cfg!(windows) {
        include_paths.push("Kinc/Backends/System/Microsoft/Sources");
        include_paths.push("Kinc/Backends/System/Windows/Sources")
    }
    if cfg!(target_vendor = "apple") {
        include_paths.push("Kinc/Backends/System/Apple/Sources")
    }
    if cfg!(target_os = "macos") {
        include_paths.push("Kinc/Backends/System/MacOS/Sources")
    }
    if cfg!(target_os = "ios") {
        include_paths.push("Kinc/Backends/System/iOS/Sources")
    }
    include_paths.append(&mut vec!["Kinc/Backends/Graphics4/OpenGL/Sources"]);
    include_paths.append(&mut vec!["Kinc/Backends/Graphics5/G5onG4/Sources"]);
    defines.push("KORE_OPENGL");
    if cfg!(windows) {
        defines.push("GLEW_STATIC");
    } else if cfg!(target_os = "linux") {
        defines.push("KINC_EGL");
    }

    dbg!(include_paths.clone());
    dbg!(defines.clone());

    let bindings = bindgen::Builder::default()
        .header("kinc.h")
        .clang_args(
            include_paths
                .iter()
                .flat_map(|p| ["-I", p])
                .chain(defines.iter().flat_map(|d| ["-D", d])),
        )
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let mut builder = cc::Build::new();
    builder.file("Kinc/Sources/kinc/rootunit.c");
    builder.file("Kinc/Sources/kinc/audio1/a1unit.c");
    builder.file("Kinc/Sources/kinc/audio2/audio.c");
    builder.file("Kinc/Sources/kinc/graphics4/g4unit.c");
    builder.file("Kinc/Sources/kinc/graphics5/g5unit.c");
    builder.file("Kinc/Sources/kinc/input/inputunit.c");
    builder.file("Kinc/Sources/kinc/io/iounit.c");
    builder.file("Kinc/Sources/kinc/math/mathunit.c");
    builder.file("Kinc/Sources/kinc/network/networkunit.c");
    if cfg!(unix) {
        builder.file("Kinc/Backends/System/POSIX/Sources/kinc/backend/posixunit.c");
    }
    if cfg!(target_os = "linux") {
        builder.file("Kinc/Backends/System/Linux/Sources/kinc/backend/linuxunit.c");
        for lib in ["asound", "dl", "udev", "GL", "EGL"] {
            println!("cargo:rustc-link-lib={}", lib);
        }
    }

    builder.file("Kinc/Backends/Graphics4/OpenGL/Sources/kinc/backend/graphics4/openglunit.c");
    #[cfg(windows)]
    builder.file("Kinc/Backends/Graphics4/OpenGL/Sources/GL/glew.c");

    for define in defines {
        builder.define(define, None);
    }
    builder.includes(include_paths);
    builder.extra_warnings(false);
    builder.cargo_metadata(true);
    builder.warnings(false);
    builder.flag_if_supported("-Wno-attributes");
    builder.compile("kinc");
}
