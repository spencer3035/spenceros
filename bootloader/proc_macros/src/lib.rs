use quote::quote;

use proc_macro2::TokenStream;
use syn::DeriveInput;

mod gen;
mod parse;

const SCAN_ATTR: &str = "scan";
const SCAN_TRAIT: &str = "FromScancodes";

#[proc_macro_derive(FromScancodes, attributes(scan))]
pub fn from_scancodes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match impl_from_scancodes(&input) {
        Ok(output) => quote! {#output}.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn impl_from_scancodes(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let enum_data = parse::expect_enum(input)?;
    let mut items = Vec::new();
    for field in enum_data.variants.iter() {
        for attr in field.attrs.iter() {
            if attr.path().is_ident(SCAN_ATTR) {
                let attr: parse::ScanKeyValues = attr.meta.require_list()?.parse_args()?;
                let item = parse::ScancodeItem {
                    name: field.ident.clone(),
                    down: attr.down,
                    up: attr.up,
                    ch: attr.ch,
                };

                items.push(item);
            }
        }
    }

    Ok(gen::gen_impl(&items, name))
}
