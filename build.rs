extern crate bindgen;

use std::env;
use std::fmt::Display;
use std::path::PathBuf;

enum GraphicsApi {
    OpenGL,
    Vulkan,
    D3D11,
    D3D12,
    Metal,
}

impl GraphicsApi {
    pub fn is_g4(&self) -> bool {
        match self {
            Self::OpenGL | Self::D3D11 => true,
            _ => false,
        }
    }

    pub fn is_supported(&self) -> bool {
        match self {
            Self::OpenGL => {
                cfg!(target_os = "linux")
                    || cfg!(target_os = "macos")
                    || cfg!(target_os = "windows")
                    || cfg!(target_os = "android")
                    || cfg!(target_os = "ios")
            }
            Self::Vulkan => {
                cfg!(target_os = "linux")
                    || cfg!(target_os = "windows")
                    || cfg!(target_os = "android")
            }
            Self::D3D11 | Self::D3D12 => cfg!(target_os = "windows"),
            Self::Metal => cfg!(target_os = "macos") || cfg!(target_os = "ios"),
        }
    }
}

impl Display for GraphicsApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenGL => write!(f, "OpenGL"),
            Self::Vulkan => write!(f, "Vulkan"),
            Self::D3D11 => write!(f, "D3D11"),
            Self::D3D12 => write!(f, "D3D12"),
            Self::Metal => write!(f, "Metal"),
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=kinc.h");

    let mut include_paths = vec!["Kinc/Sources"];
    let mut defines = vec!["KINC_NO_MAIN"];
    let mut libs = vec![];
    let mut files = vec![
        "Kinc/Sources/kinc/rootunit.c",
        "Kinc/Sources/kinc/audio1/a1unit.c",
        "Kinc/Sources/kinc/audio2/audio.c",
        "Kinc/Sources/kinc/graphics4/g4unit.c",
        "Kinc/Sources/kinc/graphics5/g5unit.c",
        "Kinc/Sources/kinc/input/inputunit.c",
        "Kinc/Sources/kinc/io/iounit.c",
        "Kinc/Sources/kinc/math/mathunit.c",
        "Kinc/Sources/kinc/network/networkunit.c",
    ];

    if cfg!(unix) {
        include_paths.push("Kinc/Backends/System/POSIX/Sources");
        files.push("Kinc/Backends/System/POSIX/Sources/kinc/backend/posixunit.c");
    }
    if cfg!(target_os = "linux") {
        include_paths.push("Kinc/Backends/System/Linux/Sources");
        defines.push("KINC_NO_WAYLAND");
        files.push("Kinc/Backends/System/Linux/Sources/kinc/backend/linuxunit.c");
        libs.extend(["asound", "dl", "udev"]);
    }
    if cfg!(windows) {
        include_paths.push("Kinc/Backends/System/Microsoft/Sources");
        files.push("Kinc/Backends/System/Microsoft/Sources/kinc/backend/microsoftunit.c");
        include_paths.push("Kinc/Backends/System/Windows/Sources");
        files.push("Kinc/Backends/System/Windows/Sources/kinc/backend/windowsunit.c");
        files.push("Kinc/Backends/System/Windows/Sources/kinc/backend/windowscppunit.cpp");
    }
    if cfg!(target_vendor = "apple") {
        include_paths.push("Kinc/Backends/System/Apple/Sources");
        files.push("Kinc/Backends/System/Apple/Sources/kinc/backend/appleunit.m");
    }
    if cfg!(target_os = "macos") {
        include_paths.push("Kinc/Backends/System/MacOS/Sources");
        files.push("Kinc/Backends/System/macOS/Sources/kinc/backend/macosunit.m");
        libs.push("framework=IOKit");
        libs.push("framework=Cocoa");
        libs.push("framework=AppKit");
        libs.push("framework=CoreAudio");
        libs.push("framework=CoreData");
        libs.push("framework=CoreMedia");
        libs.push("framework=CoreVideo");
        libs.push("framework=AVFoundation");
        libs.push("framework=Foundation");
    }
    if cfg!(target_os = "ios") {
        include_paths.push("Kinc/Backends/System/iOS/Sources");
        files.push("Kinc/Backends/System/iOS/Sources/kinc/backend/iosunit.m");
    }

    let default_graphics = if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
        GraphicsApi::Metal
    } else if cfg!(target_os = "windows") {
        GraphicsApi::D3D11
    } else {
        GraphicsApi::OpenGL
    };

    let graphics = if cfg!(feature = "opengl") {
        Some(GraphicsApi::OpenGL)
    } else if cfg!(feature = "vulkan") {
        Some(GraphicsApi::Vulkan)
    } else if cfg!(feature = "d3d11") {
        Some(GraphicsApi::D3D11)
    } else if cfg!(feature = "d3d12") {
        Some(GraphicsApi::D3D12)
    } else if cfg!(feature = "metal") {
        Some(GraphicsApi::Metal)
    } else {
        None
    };

    let graphics = graphics.unwrap_or(default_graphics);

    if !graphics.is_supported() {
        panic!("{} is not supported on this target", graphics);
    }

    if graphics.is_g4() {
        defines.push("KORE_G4");
        defines.push("KORE_G5ONG4");
        include_paths.push("Kinc/Backends/Graphics5/G5onG4/Sources");
        files.push("Kinc/Backends/Graphics5/G5onG4/Sources/kinc/backend/graphics5/g5ong4unit.c");
    } else {
        defines.push("KORE_G5");
        defines.push("KORE_G4ONG5");
        include_paths.push("Kinc/Backends/Graphics4/G4onG5/Sources");
        files.push("Kinc/Backends/Graphics4/G4onG5/Sources/kinc/backend/graphics4/g4ong5unit.c");
    }

    match graphics {
        GraphicsApi::OpenGL => {
            include_paths.push("Kinc/Backends/Graphics4/OpenGL/Sources");
            defines.push("KORE_OPENGL");
            files
                .push("Kinc/Backends/Graphics4/OpenGL/Sources/kinc/backend/graphics4/openglunit.c");
            if cfg!(windows) {
                defines.push("GLEW_STATIC");
                files.push("Kinc/Backends/Graphics4/OpenGL/Sources/GL/glew.c");
            }
            if cfg!(target_os = "macos") {
                libs.push("framework=OpenGL");
            }
            if cfg!(target_os = "linux") || cfg!(target_os = "android") {
                defines.push("KINC_EGL");
                libs.extend(["GL", "EGL"]);
            }
            if cfg!(target_os = "android") || cfg!(target_os = "android") {
                defines.push("KORE_OPENGL_ES");
            }
        }
        GraphicsApi::Vulkan => {
            include_paths.push("Kinc/Backends/Graphics5/Vulkan/Sources");
            defines.push("KORE_VULKAN");
            files.push("Kinc/Backends/Graphics5/Vulkan/Sources/kinc/backend/compute.c");
            files
                .push("Kinc/Backends/Graphics5/Vulkan/Sources/kinc/backend/graphics5/vulkanunit.c");
            libs.push("vulkan");
        }
        GraphicsApi::D3D11 => {
            include_paths.push("Kinc/Backends/Graphics4/Direct3D11/Sources");
            defines.push("KORE_D3D11");
            files.push("Kinc/Backends/Graphics4/Direct3D11/Sources/kinc/backend/compute.c");
            files.push(
                "Kinc/Backends/Graphics4/Direct3D11/Sources/kinc/backend/graphics4/d3d11unit.c",
            );
        }
        GraphicsApi::D3D12 => {
            include_paths.push("Kinc/Backends/Graphics5/Direct3D12/Sources");
            defines.push("KORE_D3D12");
            files.push(
                "Kinc/Backends/Graphics5/Direct3D12/Sources/kinc/backend/graphics5/d3d12unit.c",
            );
        }
        GraphicsApi::Metal => {
            include_paths.push("Kinc/Backends/Graphics5/Metal/Sources");
            defines.push("KORE_METAL");
            files.push("Kinc/Backends/Graphics5/Metal/Sources/kinc/backend/compute.m");
            files.push("Kinc/Backends/Graphics5/Metal/Sources/kinc/backend/graphics5/metalunit.m");
            libs.extend(["framework=Metal"]);
        }
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

    builder.files(files);

    for lib in libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    for define in defines {
        builder.define(define, None);
    }
    builder.includes(include_paths);
    builder.extra_warnings(false);
    builder.cargo_metadata(true);
    builder.warnings(false);
    builder.flag_if_supported("-Wno-attributes");
    // builder.cpp(true);
    builder.compile("kinc");
}
