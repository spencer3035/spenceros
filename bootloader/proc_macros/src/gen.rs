use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeMap;
use syn::Ident;

use crate::parse::ScancodeItem;
use crate::SCAN_TRAIT;

pub(crate) fn gen_impl(items: &[ScancodeItem], name: &Ident) -> TokenStream {
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
pub(crate) fn has_next(code: u8, index: u8) -> bool {
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

pub(crate) fn gen_has_next(items: &[ScancodeItem]) -> TokenStream {
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

pub(crate) fn gen_to_char(items: &[ScancodeItem]) -> TokenStream {
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

pub(crate) enum KeyCode {
    Kc1,
    Kc2,
    Kc3,
    Kc4,
}

#[allow(dead_code)]
pub(crate) fn from_scancode_and_depth(code: u8, idx: u8) -> Option<(KeyCode, bool)> {
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

pub(crate) fn gen_from_scancode_and_depth(items: &[ScancodeItem]) -> TokenStream {
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
