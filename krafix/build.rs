use std::env;

fn find_files_rec(path: String, cb: &mut dyn FnMut(&str)) {
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            // eprintln!("{:?}", path);
            find_files_rec(path.to_str().unwrap().to_owned(), cb)
        } else {
            cb(path.to_str().unwrap());
        }
    }
}

fn main() {
    let include_paths = vec![
        "krafix/glslang",
        "krafix/glslang/glslang",
        "krafix/glslang/glslang/MachineIndependent",
        "krafix/glslang/glslang/Include",
        "krafix/glslang/OGLCompilersDLL",
        "krafix/SPIRV-Cross",
    ];
    let defines = vec![
        "KRAFIX_LIBRARY",
        "SPIRV_CROSS_KRAFIX",
        "ENABLE_HLSL",
        "NV_EXTENSIONS",
        "AMD_EXTENSIONS",
    ];

    let target_is_window = env::var("CARGO_CFG_TARGET_OS")
        .unwrap()
        .split('-')
        .next()
        .unwrap()
        == "windows";

    let mut builder = cc::Build::new();

    let mut files = |pat: &str| {
        if pat.ends_with("**") {
            find_files_rec(pat[0..pat.len() - 2].to_owned(), &mut |f| {
                if f.ends_with("cpp") || f.ends_with('c') {
                    builder.file(f);
                }
            });
        } else if pat.ends_with("*") {
            for f in std::fs::read_dir(&pat[0..pat.len() - 1]).unwrap() {
                let f = f.unwrap().path();
                let f = f.to_str().unwrap();
                if f.ends_with("cpp") || f.ends_with('c') {
                    builder.file(f);
                }
            }
        } else {
            builder.file(pat);
        }
    };

    for pat in [
        "Sources/**",
        "sourcemap.cpp/src/**",
        "sourcemap.cpp/deps/json/json.cpp",
        "sourcemap.cpp/deps/cencode/cencode.c",
        "sourcemap.cpp/deps/cencode/cdecode.c",
        "sourcemap.cpp/src/map_line.cpp",
        "sourcemap.cpp/src/map_col.cpp",
        "sourcemap.cpp/src/mappings.cpp",
        "sourcemap.cpp/src/pos_idx.cpp",
        "sourcemap.cpp/src/pos_txt.cpp",
        "sourcemap.cpp/src/format/v3.cpp",
        "sourcemap.cpp/src/document.cpp",
        "glslang/StandAlone/ResourceLimits.cpp",
        "glslang/glslang/GenericCodeGen/**",
        "glslang/glslang/MachineIndependent/**",
        "glslang/glslang/Include/**",
        "glslang/hlsl/**",
        "glslang/OGLCompilersDLL/**",
        "glslang/SPIRV/**",
        "SPIRV-Cross/*",
    ] {
        files(&format!("krafix/{}", pat));
    }

    if target_is_window {
        files("krafix/glslang/glslang/OSDependent/Windows/**");
        builder.include("krafix/glslang/glslang/OSDependent/Windows");
        println!("cargo:rustc-link-lib=d3dcompiler");
    } else {
        files("krafix/glslang/glslang/OSDependent/Unix/**");
        builder.include("krafix/glslang/glslang/OSDependent/Unix");
    }

    builder.cpp(true);
    builder.flag_if_supported("-std=c++11");

    for define in defines {
        builder.define(define, None);
    }

    builder.includes(include_paths);
    builder.extra_warnings(false);
    builder.cargo_metadata(true);
    builder.warnings(false);
    // builder.flag_if_supported("-Wno-attributes");
    builder.compile("krafix");
}
