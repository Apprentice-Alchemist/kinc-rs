use std::ffi::CString;

use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident, LitByteStr, LitStr, Token,
};

enum ShaderKind {
    Vertex,
    Fragment,
}

struct Shader {
    kind: ShaderKind,
    lang: String,
    source: String,
}

extern "C" {
    fn krafix_compile(
        source: *const u8,
        output: *mut u8,
        length: *mut i32,
        targetlang: *const u8,
        system: *const u8,
        shadertype: *const u8,
        shaderversion: i32,
    ) -> i32;
}

impl Parse for Shader {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let lang = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let source: LitStr = input.parse()?;
        let shader_code = source.value();

        let kind = match kind.to_string().as_str() {
            "vertex" => ShaderKind::Vertex,
            "fragment" => ShaderKind::Fragment,
            _ => panic!("Unknown shader kind"),
        };

        Ok(Shader {
            kind,
            lang: lang.to_string(),
            source: shader_code,
        })
    }
}

#[proc_macro]
pub fn compile_shader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let shader: Shader = parse_macro_input!(input as Shader);

    let source = CString::new(shader.source).unwrap();
    let targetlang = CString::new(shader.lang).unwrap();
    let system = if cfg!(windows) {
        &b"windows\0"[..]
    } else if cfg!(target_os = "macos") {
        &b"macos\0"[..]
    } else {
        &b"linux\0"[..]
    };

    let shadertype = match shader.kind {
        ShaderKind::Vertex => b"vert\0",
        ShaderKind::Fragment => b"frag\0",
    };

    let shaderversion = 300;

    let mut v = vec![0_u8; 1024 * 1024];
    let mut len: i32 = v.len() as i32;

    unsafe {
        if krafix_compile(
            source.as_ptr().cast(),
            v.as_mut_ptr(),
            &mut len as *mut i32,
            targetlang.as_ptr().cast(),
            system.as_ptr(),
            shadertype.as_ptr(),
            shaderversion
        ) != 0
        {
            panic!("Failed to compile shader");
        }
    }

    let byte_string = LitByteStr::new(&v[0..(len as usize)], Span::call_site());
    quote! {
        #byte_string
    }
    .into()
}
