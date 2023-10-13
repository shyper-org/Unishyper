extern crate proc_macro;

use proc_macro::*;
use quote::quote;
use quote::ToTokens;

#[proc_macro_attribute]
pub fn panic_inject(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: syn::Item = syn::parse(input).unwrap();
    let fn_item = match &mut item {
        syn::Item::Fn(fn_item) => fn_item,
        _ => panic!("This attribute only targets function"),
    };
    let statements = &mut fn_item.block.stmts;
    let ident = &fn_item.sig.ident;
    let id = ident.to_string();
    if let Some(x) = option_env!("PI") {
        println!("panic_inject {}: {}", x, id);
        if x == id {
            let len = statements.len();
            let at = rand::random::<usize>() % len;
            statements.insert(
                at,
                syn::parse(quote!(crate::random_panic();).into()).unwrap(),
            );
            println!("panic_inject {}: {} of {}", ident, at, len);
        }
    } else {
        println!("PI not found");
    }
    item.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn count_stmts(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item: syn::Item = syn::parse(input).unwrap();
    let fn_item = match &item {
        syn::Item::Fn(fn_item) => fn_item,
        _ => panic!("This attribute only targets function"),
    };
    let statements = &fn_item.block.stmts;
    let len = statements.len();
    let ident = &fn_item.sig.ident;
    println!("count_stmts of {}: {}", ident, len);
    item.into_token_stream().into()
}
