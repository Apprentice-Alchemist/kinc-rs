use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, Ident, Token, Type, Visibility, LitStr};

enum ShaderKind {
    Vertex,
    Fragment,
}

struct Shader {
    kind: ShaderKind,
    source: String,
}

impl Parse for Shader {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let source: LitStr = input.parse()?;
        let shader_code = source.value();

        let kind = match kind.to_string().as_str() {
            "vertex" => ShaderKind::Vertex,
            "fragment" => ShaderKind::Fragment,
            _ => panic!("Unknown shader kind"),
        };

        Ok(Shader {
            kind,
            source: shader_code,
        })
    }
}

#[proc_macro]
pub fn shader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let shader: Shader = parse_macro_input!(input as Shader);

    // std::process::Command::new("krafix")
    //     .arg(shader.kind.to_string())
    //     .arg(shader.source)
    //     .output()
    //     .expect("Failed to run shader command");

    quote! {
        ()
    }.into()
}
