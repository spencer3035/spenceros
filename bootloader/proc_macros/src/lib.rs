use std::collections::BTreeMap;

use proc_macro2::Span;
use quote::quote;

use proc_macro2::{Literal, TokenStream};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    DataEnum, DeriveInput, Error, Ident, Token,
};

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
    let enum_data = expect_enum(input)?;
    let mut items = Vec::new();
    for field in enum_data.variants.iter() {
        for attr in field.attrs.iter() {
            if attr.path().is_ident(SCAN_ATTR) {
                let attr: ScanKeyValues = attr.meta.require_list()?.parse_args()?;
                let item = ScancodeItem {
                    name: field.ident.clone(),
                    down: attr.down,
                    up: attr.up,
                    ch: attr.ch,
                };

                items.push(item);
            }
        }
    }

    Ok(gen_impl(&items, name))
}

fn gen_impl(items: &[ScancodeItem], name: &Ident) -> TokenStream {
    let has_next = gen_has_next(items);
    let to_char = gen_to_char(items);
    let from_scancode_and_depth = gen_from_scancode_and_depth(items);
    let trait_name = Ident::new(SCAN_TRAIT, Span::call_site());
    quote!(
        impl #trait_name for #name {
            #has_next
            #to_char
            #from_scancode_and_depth
        }
    )
}

#[allow(dead_code)]
fn has_next(code: u8, index: u8) -> bool {
    if false {
        false
    } else if index == 0 {
        false || code == 0x01 || code == 0x02 || code == 0x03
    } else if index == 1 {
        false || code == 0x04 || code == 0x05 || code == 0x06
    } else {
        false
    }
}

fn gen_has_next(items: &[ScancodeItem]) -> TokenStream {
    let mut impls: BTreeMap<u8, TokenStream> = BTreeMap::new();
    for kc in items.iter() {
        for (code_ii, code_win) in kc.down.windows(2).enumerate() {
            let code_curr = &code_win[0];
            let or_code = quote! { || code == #code_curr };
            if let Some(imp) = impls.get_mut(&(code_ii as u8)) {
                imp.extend(or_code);
            } else {
                let val = quote! { false #or_code };
                impls.insert(code_ii as u8, val);
            }
        }
        for (code_ii, code_win) in kc.up.windows(2).enumerate() {
            let code_curr = &code_win[0];
            let or_code = quote! { || code == #code_curr };
            if let Some(imp) = impls.get_mut(&(code_ii as u8)) {
                imp.extend(or_code);
            } else {
                let val = quote! { false #or_code };
                impls.insert(code_ii as u8, val);
            }
        }
    }
    let mut blocks = quote! {
        if false {
            false
        }
    };
    for (idx, imp) in impls.into_iter() {
        let block = quote! {
            else if idx == #idx  {
                #imp
            }
        };
        blocks.extend(block);
    }
    blocks.extend(quote! {
        else {
            false
        }
    });
    quote! {
        fn has_next(code: u8, idx : u8) -> bool {
            #blocks
        }
    }
}

fn gen_to_char(items: &[ScancodeItem]) -> TokenStream {
    let names = items.iter().map(|i| &i.name);
    // let chars = items.iter().map(|i| &i.ch);
    let chars = items.iter().map(|i| match &i.ch {
        Some(ch) => quote!(Some(#ch)),
        None => quote!(None),
    });
    quote! {
        fn to_char(&self) -> Option<char> {
            match self {
            #(
                Self::#names => #chars,
            )*
            }
        }
    }
}

enum KeyCode {
    Kc1,
    Kc2,
    Kc3,
    Kc4,
}

#[allow(dead_code)]
fn from_scancode_and_depth(code: u8, idx: u8) -> Option<(KeyCode, bool)> {
    if false {
        None
    } else if idx == 0 {
        if false {
            None
        } else if code == 0x01 {
            Some((KeyCode::Kc1, true))
        } else if code == 0x02 {
            Some((KeyCode::Kc2, true))
        } else {
            None
        }
    } else if idx == 1 {
        if false {
            None
        } else if code == 0x03 {
            Some((KeyCode::Kc3, true))
        } else if code == 0x04 {
            Some((KeyCode::Kc4, true))
        } else {
            None
        }
    } else {
        None
    }
}

fn gen_from_scancode_and_depth(items: &[ScancodeItem]) -> TokenStream {
    let mut impls: BTreeMap<u8, TokenStream> = BTreeMap::new();
    for kc in items.iter() {
        let name = &kc.name;
        // Down
        for (code_ii, code_curr) in kc.down.iter().enumerate() {
            let else_if = quote! {
                else if code == #code_curr {
                    Some((Self::#name, true))
                }
            };
            if let Some(imp) = impls.get_mut(&(code_ii as u8)) {
                imp.extend(else_if);
            } else {
                let val = quote! { if false { None } #else_if };
                impls.insert(code_ii as u8, val);
            }
        }
        // Up
        for (code_ii, code_curr) in kc.up.iter().enumerate() {
            let else_if = quote! {
                else if code == #code_curr {
                    Some((Self::#name, false))
                }
            };
            if let Some(imp) = impls.get_mut(&(code_ii as u8)) {
                imp.extend(else_if);
            } else {
                let val = quote! { if false { None } #else_if };
                impls.insert(code_ii as u8, val);
            }
        }
    }
    for (_key, imp) in impls.iter_mut() {
        imp.extend(quote! {
            else {
                None
            }
        });
    }
    let mut blocks = quote! {
        if false {
            None
        }
    };
    for (idx, imp) in impls.into_iter() {
        let block = quote! {
            else if idx == #idx  {
                #imp
            }
        };
        blocks.extend(block);
    }
    blocks.extend(quote! {
        else {
            None
        }
    });
    quote! {
        fn from_scancode_and_depth(code: u8, idx : u8) -> Option<(Self, bool)> {
            #blocks
        }
    }
}

struct ScancodeItem {
    name: Ident,
    down: Vec<Literal>,
    up: Vec<Literal>,
    ch: Option<Literal>,
}

struct KeyValue<T: Parse> {
    key: Ident,
    _eq: Token![=],
    value: T,
}

impl<T: Parse> Parse for KeyValue<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

struct ScanKeyValues {
    down: Vec<Literal>,
    up: Vec<Literal>,
    ch: Option<Literal>,
}

impl Parse for ScanKeyValues {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Down
        let kv: KeyValue<BracketedList<Literal>> = input.parse()?;
        if kv.key != "down" {
            return Err(input.error("missing 'down'"));
        }
        let down = kv.value.list.into_iter().collect();
        _ = input.parse::<Token![,]>()?;
        // Up
        let kv: KeyValue<BracketedList<Literal>> = input.parse()?;
        if kv.key != "up" {
            return Err(input.error("missing 'up'"));
        }
        let up = kv.value.list.into_iter().collect();
        // Optional char
        let mut ch = None;
        if input.parse::<Token![,]>().is_ok() {
            if let Ok(kv) = input.parse::<KeyValue<Literal>>() {
                if kv.key != "ch" {
                    return Err(input.error("missing 'ch'"));
                } else {
                    ch = Some(kv.value)
                }
            }
        }

        Ok(ScanKeyValues { down, up, ch })
    }
}

struct BracketedList<T> {
    _bracket: syn::token::Bracket,
    list: Punctuated<T, Token![,]>,
}

impl<T: Parse> Parse for BracketedList<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let bracket = syn::bracketed!(content in input);
        Ok(BracketedList {
            _bracket: bracket,
            list: content.call(Punctuated::parse_terminated)?,
        })
    }
}

fn expect_enum(input: &DeriveInput) -> syn::Result<&DataEnum> {
    match &input.data {
        syn::Data::Enum(e) => Ok(e),
        _ => Err(Error::new_spanned(
            &input.ident,
            "FromScancodes can only be derived for enums",
        )),
    }
}
