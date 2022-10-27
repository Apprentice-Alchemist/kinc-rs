#![allow(clippy::upper_case_acronyms)]

use core::panic;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

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
            Self::Metal => matches!(os, TargetOS::MacOS | TargetOS::IOS | TargetOS::TVOS),
        }
    }

    pub fn to_feature(self) -> &'static str {
        match self {
            Self::OpenGL => "opengl",
            Self::Vulkan => "vulkan",
            Self::D3D11 => "d3d11",
            Self::D3D12 => "d3d12",
            Self::Metal => "metal",
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
    let mut files = Vec::new();

    fn add(v: &mut Vec<String>, f: impl ToString) {
        v.push(f.to_string());
    }

    for f in [
        "Kinc/Sources/kinc/rootunit.c",
        "Kinc/Sources/kinc/audio1/a1unit.c",
        "Kinc/Sources/kinc/audio2/audio.c",
        "Kinc/Sources/kinc/graphics4/g4unit.c",
        "Kinc/Sources/kinc/graphics5/g5unit.c",
        "Kinc/Sources/kinc/input/inputunit.c",
        "Kinc/Sources/kinc/io/iounit.c",
        "Kinc/Sources/kinc/math/mathunit.c",
        "Kinc/Sources/kinc/network/networkunit.c",
    ] {
        add(&mut files, f);
    }

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
        add(
            &mut files,
            "Kinc/Backends/System/POSIX/Sources/kinc/backend/posixunit.c",
        );
    }

    match target_os {
        TargetOS::Windows => {
            include_paths.push("Kinc/Backends/System/Microsoft/Sources");
            add(
                &mut files,
                "Kinc/Backends/System/Microsoft/Sources/kinc/backend/microsoftunit.c",
            );
            include_paths.push("Kinc/Backends/System/Windows/Sources");
            add(
                &mut files,
                "Kinc/Backends/System/Windows/Sources/kinc/backend/windowsunit.c",
            );
            add(
                &mut files,
                "Kinc/Backends/System/Windows/Sources/kinc/backend/windowscppunit.cpp",
            );
            libs.extend([
                "dxguid", "dsound", "dinput8", "ws2_32", "Winhttp", "wbemuuid", "kernel32",
                "user32", "gdi32", "comdlg32", "advapi32", "shell32", "ole32", "oleaut32", "uuid",
                "odbc32", "odbccp32",
            ]);
            defines.extend([
                "_CRT_SECURE_NO_WARNINGS",
                "_WINSOCK_DEPRECATED_NO_WARNINGS",
                "KINC_NO_DIRECTSHOW",
                "_UNICODE",
                "UNICODE",
            ]);

            add(
                &mut files,
                "Kinc/Backends/Audio2/WASAPI/Sources/kinc/backend/wasapi.c",
            );
        }
        TargetOS::Linux => {
            include_paths.push("Kinc/Backends/System/Linux/Sources");
            let wayland_dir = Path::new(&env::var("OUT_DIR").unwrap()).join("wayland");
            {
                // TODO: this is stupid
                let s = wayland_dir.display().to_string();
                let s = Box::leak(Box::new(s));
                include_paths.push(s.as_str());
            }
            let wayland_dir = wayland_dir.join("wayland-generated");
            fs::create_dir_all(&wayland_dir).unwrap();

            let mut cmd = Command::new("wayland-scanner");
            cmd.arg("private-code");
            cmd.arg("/usr/share/wayland/wayland.xml");
            let cfile = wayland_dir.join("wayland.c").display().to_string();
            cmd.arg(&cfile);
            assert!(cmd.status().unwrap().success());
            add(&mut files, cfile);
            let mut cmd = Command::new("wayland-scanner");
            cmd.arg("client-header");
            cmd.arg("/usr/share/wayland/wayland.xml");
            cmd.arg(wayland_dir.join("wayland.h").to_str().unwrap());
            assert!(cmd.status().unwrap().success());
            for (protocol, file) in [
                ("stable/viewporter/viewporter.xml", "wayland-viewporter"),
                ("stable/xdg-shell/xdg-shell.xml", "xdg-shell"),
                (
                    "unstable/xdg-decoration/xdg-decoration-unstable-v1.xml",
                    "xdg-decoration",
                ),
                ("unstable/tablet/tablet-unstable-v2.xml", "wayland-tablet"),
                (
                    "unstable/pointer-constraints/pointer-constraints-unstable-v1.xml",
                    "wayland-pointer-constraint",
                ),
                (
                    "unstable/relative-pointer/relative-pointer-unstable-v1.xml",
                    "wayland-relative-pointer",
                ),
            ] {
                let protocol_path = Path::new("/usr/share/wayland-protocols").join(protocol);
                let mut cmd = Command::new("wayland-scanner");
                cmd.arg("private-code");
                cmd.arg(protocol_path.to_str().unwrap());
                let cfile = wayland_dir.join(file.to_string() + ".c");
                let cfile = cfile.display().to_string();
                cmd.arg(&cfile);
                assert!(cmd.status().unwrap().success());
                add(&mut files, cfile);
                let mut cmd = Command::new("wayland-scanner");
                cmd.arg("client-header");
                cmd.arg(protocol_path.to_str().unwrap());
                cmd.arg(wayland_dir.join(file.to_string() + ".h").to_str().unwrap());
                assert!(cmd.status().unwrap().success());
            }

            add(
                &mut files,
                "Kinc/Backends/System/Linux/Sources/kinc/backend/linuxunit.c",
            );
            libs.extend(["asound", "dl", "udev"]);
        }
        TargetOS::MacOS => {
            include_paths.push("Kinc/Backends/System/Apple/Sources");
            add(
                &mut files,
                "Kinc/Backends/System/Apple/Sources/kinc/backend/appleunit.m",
            );
            include_paths.push("Kinc/Backends/System/MacOS/Sources");
            add(
                &mut files,
                "Kinc/Backends/System/macOS/Sources/kinc/backend/macosunit.m",
            );
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
            add(
                &mut files,
                "Kinc/Backends/System/Apple/Sources/kinc/backend/appleunit.m",
            );
            if target_os == TargetOS::TVOS {
                defines.push("KORE_TVOS");
            }
            include_paths.push("Kinc/Backends/System/iOS/Sources");
            add(
                &mut files,
                "Kinc/Backends/System/iOS/Sources/kinc/backend/iosunit.m",
            );
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
            defines.push("KORE_ANDROID");
            include_paths.push("Kinc/Backends/System/Android/Sources");
            add(
                &mut files,
                "Kinc/Backends/System/Android/Sources/kinc/backend/androidunit.c",
            );
            add(
                &mut files,
                "Kinc/Backends/System/Android/Sources/android_native_app_glue.c",
            );
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
            add(
                &mut files,
                "Kinc/Backends/System/Web/Sources/kinc/backend/webunit.c",
            );
        }
    }

    let graphics = if env::var("CARGO_FEATURE_OPENGL").is_ok() {
        GraphicsApi::OpenGL
    } else if env::var("CARGO_FEATURE_VULKAN").is_ok() {
        GraphicsApi::Vulkan
    } else if env::var("CARGO_FEATURE_D3D11").is_ok() {
        GraphicsApi::D3D11
    } else if env::var("CARGO_FEATURE_D3D12").is_ok() {
        GraphicsApi::D3D12
    } else if env::var("CARGO_FEATURE_METAL").is_ok() {
        GraphicsApi::Metal
    } else {
        let api = match target_os {
            TargetOS::MacOS | TargetOS::IOS | TargetOS::TVOS => GraphicsApi::Metal,
            TargetOS::Windows => GraphicsApi::D3D11,
            TargetOS::Android | TargetOS::Linux | TargetOS::Web => GraphicsApi::OpenGL,
        };
        println!("cargo:rustc-cfg=feature=\"{}\"", api.to_feature());
        api
    };

    if !graphics.is_supported(target_os) {
        panic!("{} is not supported on {}", graphics, target_os);
    }

    if graphics.is_g4() {
        defines.push("KORE_G4");
        defines.push("KORE_G5ONG4");
        include_paths.push("Kinc/Backends/Graphics5/G5onG4/Sources");
        add(
            &mut files,
            "Kinc/Backends/Graphics5/G5onG4/Sources/kinc/backend/graphics5/g5ong4unit.c",
        );
    } else {
        defines.push("KORE_G5");
        defines.push("KORE_G4ONG5");
        include_paths.push("Kinc/Backends/Graphics4/G4onG5/Sources");
        add(
            &mut files,
            "Kinc/Backends/Graphics4/G4onG5/Sources/kinc/backend/graphics4/g4ong5unit.c",
        );
    }

    match graphics {
        GraphicsApi::OpenGL => {
            include_paths.push("Kinc/Backends/Graphics4/OpenGL/Sources");
            defines.push("KORE_OPENGL");
            add(
                &mut files,
                "Kinc/Backends/Graphics4/OpenGL/Sources/kinc/backend/graphics4/openglunit.c",
            );
            add(
                &mut files,
                "Kinc/Backends/Graphics4/OpenGL/Sources/kinc/backend/compute.c",
            );
            match target_os {
                TargetOS::Windows => {
                    defines.push("GLEW_STATIC");
                    add(
                        &mut files,
                        "Kinc/Backends/Graphics4/OpenGL/Sources/GL/glew.c",
                    );
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
            add(
                &mut files,
                "Kinc/Backends/Graphics5/Vulkan/Sources/kinc/backend/compute.c",
            );
            add(
                &mut files,
                "Kinc/Backends/Graphics5/Vulkan/Sources/kinc/backend/graphics5/vulkanunit.c",
            );
            libs.push("vulkan");
            if target_os == TargetOS::Android {
                defines.push("VK_USE_PLATFORM_ANDROID_KHR");
            }
        }
        GraphicsApi::D3D11 => {
            include_paths.push("Kinc/Backends/Graphics4/Direct3D11/Sources");
            defines.push("KORE_D3D11");
            defines.push("KORE_D3D");
            add(
                &mut files,
                "Kinc/Backends/Graphics4/Direct3D11/Sources/kinc/backend/compute.c",
            );
            add(
                &mut files,
                "Kinc/Backends/Graphics4/Direct3D11/Sources/kinc/backend/graphics4/d3d11unit.c",
            );
            libs.push("d3d11");
        }
        GraphicsApi::D3D12 => {
            include_paths.push("Kinc/Backends/Graphics5/Direct3D12/Sources");
            defines.push("KORE_D3D12");
            defines.push("KORE_D3D");
            add(
                &mut files,
                "Kinc/Backends/Graphics5/Direct3D12/Sources/kinc/backend/graphics5/d3d12unit.cpp",
            );
            libs.extend(["dxgi", "d3d12"]);
        }
        GraphicsApi::Metal => {
            include_paths.push("Kinc/Backends/Graphics5/Metal/Sources");
            defines.push("KORE_METAL");
            add(
                &mut files,
                "Kinc/Backends/Graphics5/Metal/Sources/kinc/backend/compute.m",
            );
            add(
                &mut files,
                "Kinc/Backends/Graphics5/Metal/Sources/kinc/backend/graphics5/metalunit.m",
            );
            libs.push("framework=Metal");
            if target_os == TargetOS::MacOS {
                libs.push("framework=MetalKit");
            }
        }
    }

    let bindings = {
        let mut builder = bindgen::Builder::default().header("kinc.h").clang_args(
            include_paths
                .iter()
                .flat_map(|p| ["-I", p])
                .chain(defines.iter().flat_map(|d| ["-D", d])),
        );
        if target_os == TargetOS::IOS {
            let mut cmd = Command::new("xcrun");
            cmd.args(["--show-sdk-path", "--sdk", "iphoneos"]);
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
            let child = cmd
                .spawn()
                .expect("'xcrun --show-sdk-path --sdk iphoneos' failed");
            let output = child.wait_with_output().unwrap();
            match String::from_utf8(output.stdout) {
                Ok(p) => builder = builder.clang_arg(format!("--sysroot={}", p.trim())),
                Err(_) => panic!("xcrun returned invalid sdk path"),
            }
        }
        if target_os == TargetOS::Android {
            builder = builder.clang_arg(&format!(
                "--sysroot={}/toolchains/llvm/prebuilt/{}-{}/sysroot",
                std::env::var("ANDROID_NDK_ROOT").unwrap(),
                {
                    #[cfg(windows)]
                    {
                        "windows"
                    }
                    #[cfg(target_os = "linux")]
                    {
                        "linux"
                    }
                    #[cfg(target_os = "macos")]
                    {
                        "darwin"
                    }
                    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
                    compile_error!("Unsupported host os for android compilation");
                },
                {
                    #[cfg(target_arch = "x86_64")]
                    {
                        "x86_64"
                    }
                    #[cfg(target_arch = "x86")]
                    {
                        "x86"
                    }
                    #[cfg(target_arch = "aarch64")]
                    {
                        "aarch64"
                    }
                    #[cfg(not(any(
                        target_arch = "x86_64",
                        target_arch = "x86",
                        target_arch = "aarch64"
                    )))]
                    compile_error!("Unsupported host architecture for android compilation");
                }
            ));
        }
        builder
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .use_core()
            .ctypes_prefix("::core::ffi")
            .generate()
            .expect("Unable to generate bindings")
    };

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
        builder.flag_if_supported(&format!(
            "--sysroot={}/toolchains/llvm/prebuilt/{}-{}/sysroot",
            std::env::var("ANDROID_NDK_ROOT").unwrap(),
            {
                #[cfg(windows)]
                {
                    "windows"
                }
                #[cfg(target_os = "linux")]
                {
                    "linux"
                }
                #[cfg(target_os = "macos")]
                {
                    "darwin"
                }
                #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
                compile_error!("Unsupported host os for android compilation");
            },
            {
                #[cfg(target_arch = "x86_64")]
                {
                    "x86_64"
                }
                #[cfg(target_arch = "x86")]
                {
                    "x86"
                }
                #[cfg(target_arch = "aarch64")]
                {
                    "aarch64"
                }
                #[cfg(not(any(
                    target_arch = "x86_64",
                    target_arch = "x86",
                    target_arch = "aarch64"
                )))]
                compile_error!("Unsupported host architecture for android compilation");
            }
        ));
    }
    builder.includes(include_paths);
    builder.extra_warnings(false);
    builder.cargo_metadata(true);
    builder.warnings(false);
    builder.flag_if_supported("-Wno-attributes");
    builder.compile("kinc");
}
