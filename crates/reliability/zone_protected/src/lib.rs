extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Error, ItemFn, ItemStatic};

#[proc_macro_attribute]
pub fn privileged_func(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return Error::new(
            Span::call_site(),
            "expect an empty attribute: `#[privileged_func]`",
        )
        .to_compile_error()
        .into();
    }

    let func = syn::parse_macro_input!(item as ItemFn);
    let func_vis = func.vis; // like pub
    let func_block = &func.block; // { some statement or expression here }

    let func_decl = func.sig;

    quote! {
        #func_vis #func_decl {
            use zone;
            let ori_zone = zone::switch_to_privilege();
            {
                #func_block
            }
            zone::switch_from_privilege(ori_zone);
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn global_var(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return Error::new(
            Span::call_site(),
            "expect an empty attribute: `#[global_var]`",
        )
        .to_compile_error()
        .into();
    }

    let declaration = syn::parse_macro_input!(item as ItemStatic);

    quote! {
        #[link_section = ".protected_data"]
        #declaration
    }
    .into()
}
