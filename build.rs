#![allow(clippy::upper_case_acronyms)]

use std::env;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum GraphicsApi {
    OpenGL,
    Vulkan,
    D3D11,
    D3D12,
    Metal,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum TargetOS {
    Windows,
    Linux,
    MacOS,
    IOS,
    TVOS,
    Android,
    Web,
}

impl Display for TargetOS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetOS::Windows => write!(f, "windows"),
            TargetOS::Linux => write!(f, "linux"),
            TargetOS::MacOS => write!(f, "macos"),
            TargetOS::IOS => write!(f, "ios"),
            TargetOS::TVOS => write!(f, "tvos"),
            TargetOS::Android => write!(f, "android"),
            TargetOS::Web => write!(f, "web"),
        }
    }
}

impl GraphicsApi {
    pub fn is_g4(&self) -> bool {
        matches!(self, Self::OpenGL | Self::D3D11)
    }

    pub fn is_supported(&self, os: TargetOS) -> bool {
        match self {
            Self::OpenGL => true, // currently OpenGL is always supported,
            Self::Vulkan => matches!(os, TargetOS::Windows | TargetOS::Linux | TargetOS::Android),
            Self::D3D11 | Self::D3D12 => os == TargetOS::Windows,
            Self::Metal => matches!(os, TargetOS::MacOS | TargetOS::IOS | TargetOS::TVOS)
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
    let mut defines = vec!["KINC_NO_MAIN", "KORE_LZ4X"];
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

    let target_os = match env::var("CARGO_CFG_TARGET_OS")
        .unwrap()
        .split('-')
        .next()
        .unwrap()
    {
        "windows" => TargetOS::Windows,
        "linux" => TargetOS::Linux,
        "macos" => TargetOS::MacOS,
        "ios" => TargetOS::IOS,
        "tvos" => TargetOS::TVOS,
        "android" => TargetOS::Android,
        "web" => TargetOS::Web,
        os => panic!("Unknown target OS: {}", os),
    };

    if env::var("CARGO_CFG_UNIX").is_ok() {
        include_paths.push("Kinc/Backends/System/POSIX/Sources");
        files.push("Kinc/Backends/System/POSIX/Sources/kinc/backend/posixunit.c");
    }

    match target_os {
        TargetOS::Windows => {
            include_paths.push("Kinc/Backends/System/Microsoft/Sources");
            files.push("Kinc/Backends/System/Microsoft/Sources/kinc/backend/microsoftunit.c");
            include_paths.push("Kinc/Backends/System/Windows/Sources");
            files.push("Kinc/Backends/System/Windows/Sources/kinc/backend/windowsunit.c");
            files.push("Kinc/Backends/System/Windows/Sources/kinc/backend/windowscppunit.cpp");
            libs.extend([
                "dxguid", "dsound", "dinput8", "ws2_32", "Winhttp", "wbemuuid", "kernel32",
                "user32", "gdi32", "comdlg32", "advapi32", "shell32", "ole32", "oleaut32", "uuid",
                "odbc32", "odbccp32",
            ]);
            defines.extend([
                "_CRT_SECURE_NO_WARNINGS",
                "_WINSOCK_DEPRECATED_NO_WARNINGS",
                "KINC_NO_DIRECTSHOW",
            ]);

            files.push("Kinc/Backends/Audio2/WASAPI/Sources/kinc/backend/wasapi.c");
        }
        TargetOS::Linux => {
            include_paths.push("Kinc/Backends/System/Linux/Sources");
            defines.push("KINC_NO_WAYLAND");
            files.push("Kinc/Backends/System/Linux/Sources/kinc/backend/linuxunit.c");
            libs.extend(["asound", "dl", "udev"]);
        }
        TargetOS::MacOS => {
            include_paths.push("Kinc/Backends/System/Apple/Sources");
            files.push("Kinc/Backends/System/Apple/Sources/kinc/backend/appleunit.m");
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
        TargetOS::IOS | TargetOS::TVOS => {
            include_paths.push("Kinc/Backends/System/Apple/Sources");
            files.push("Kinc/Backends/System/Apple/Sources/kinc/backend/appleunit.m");
            if target_os == TargetOS::TVOS {
                defines.push("KORE_TVOS");
            }
            include_paths.push("Kinc/Backends/System/iOS/Sources");
            files.push("Kinc/Backends/System/iOS/Sources/kinc/backend/iosunit.m");
            libs.push("framework=UIKit");
            libs.push("framework=Foundation");
            libs.push("framework=CoreGraphics");
            libs.push("framework=QuartzCore");
            libs.push("framework=CoreAudio");
            libs.push("framework=AudioToolbox");
            libs.push("framework=CoreMotion");
            libs.push("framework=AVFoundation");
            libs.push("framework=CoreFoundation");
            libs.push("framework=CoreVideo");
            libs.push("framework=CoreMedia");
        }
        TargetOS::Android => {
            include_paths.push("Kinc/Backends/System/Android/Sources");
            files.push("Kinc/Backends/System/Android/Sources/kinc/backend/androidunit.c");
            libs.push("log");
            libs.push("android");
            libs.push("EGL");
            libs.push("GLESv2");
            libs.push("OpenSLES");
            libs.push("OpenMAXAL");
        }
        TargetOS::Web => {
            defines.push("KORE_HTML5");
            include_paths.push("Kinc/Backends/System/HTML5/Sources");
            files.push("Kinc/Backends/System/Web/Sources/kinc/backend/webunit.c");
        }
    }

    let graphics = if env::var("CARGO_FEATURE_opengl").is_ok() {
        GraphicsApi::OpenGL
    } else if env::var("CARGO_FEATURE_vulkan").is_ok() {
        GraphicsApi::Vulkan
    } else if env::var("CARGO_FEATURE_d3d11").is_ok() {
        GraphicsApi::D3D11
    } else if env::var("CARGO_FEATURE_d3d12").is_ok() {
        GraphicsApi::D3D12
    } else if env::var("CARGO_FEATURE_metal").is_ok() {
        GraphicsApi::Metal
    } else {
        match target_os {
            TargetOS::MacOS | TargetOS::IOS | TargetOS::TVOS => GraphicsApi::Metal,
            TargetOS::Windows => GraphicsApi::D3D11,
            TargetOS::Android | TargetOS::Linux | TargetOS::Web => GraphicsApi::OpenGL,
        }
    };

    if !graphics.is_supported(target_os) {
        panic!("{} is not supported on {}", graphics, target_os);
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
            match target_os {
                TargetOS::Windows => {
                    defines.push("GLEW_STATIC");
                    files.push("Kinc/Backends/Graphics4/OpenGL/Sources/GL/glew.c");
                    libs.push("opengl32");
                }
                TargetOS::Linux => {
                    defines.push("KINC_EGL");
                    libs.extend(["GL", "EGL"]);
                }
                TargetOS::MacOS => {
                    libs.push("OpenGL");
                }
                TargetOS::IOS | TargetOS::TVOS => {
                    defines.push("KORE_OPENGL_ES");
                    libs.push("OpenGLES");
                }
                TargetOS::Android => {
                    defines.push("KINC_EGL");
                    defines.push("KORE_OPENGL_ES");
                    libs.push("GLESv2");
                }
                TargetOS::Web => {
                    defines.push("KORE_OPENGL_ES");
                }
            }
        }
        GraphicsApi::Vulkan => {
            include_paths.push("Kinc/Backends/Graphics5/Vulkan/Sources");
            defines.push("KORE_VULKAN");
            files.push("Kinc/Backends/Graphics5/Vulkan/Sources/kinc/backend/compute.c");
            files
                .push("Kinc/Backends/Graphics5/Vulkan/Sources/kinc/backend/graphics5/vulkanunit.c");
            libs.push("vulkan");
            if target_os == TargetOS::Android {
                defines.push("VK_USE_PLATFORM_ANDROID_KHR");
            }
        }
        GraphicsApi::D3D11 => {
            include_paths.push("Kinc/Backends/Graphics4/Direct3D11/Sources");
            defines.push("KORE_D3D11");
            defines.push("KORE_D3D");
            files.push("Kinc/Backends/Graphics4/Direct3D11/Sources/kinc/backend/compute.c");
            files.push(
                "Kinc/Backends/Graphics4/Direct3D11/Sources/kinc/backend/graphics4/d3d11unit.c",
            );
            libs.push("d3d11");
        }
        GraphicsApi::D3D12 => {
            include_paths.push("Kinc/Backends/Graphics5/Direct3D12/Sources");
            defines.push("KORE_D3D12");
            defines.push("KORE_D3D");
            files.push(
                "Kinc/Backends/Graphics5/Direct3D12/Sources/kinc/backend/graphics5/d3d12unit.c",
            );
            libs.extend(["dxgi", "d3d12"]);
        }
        GraphicsApi::Metal => {
            include_paths.push("Kinc/Backends/Graphics5/Metal/Sources");
            defines.push("KORE_METAL");
            files.push("Kinc/Backends/Graphics5/Metal/Sources/kinc/backend/compute.m");
            files.push("Kinc/Backends/Graphics5/Metal/Sources/kinc/backend/graphics5/metalunit.m");
            libs.push("framework=Metal");
            if target_os == TargetOS::MacOS {
                libs.push("framework=MetalKit");
            }
        }
    }

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
    if target_os == TargetOS::Android {
        if graphics == GraphicsApi::Vulkan {
            builder.define("KORE_ANDROID_API", "24");
        } else {
            builder.define("KORE_ANDROID_API", "19");
        }
    }
    builder.includes(include_paths);
    builder.extra_warnings(false);
    builder.cargo_metadata(true);
    builder.warnings(false);
    builder.flag_if_supported("-Wno-attributes");
    builder.compile("kinc");
}
