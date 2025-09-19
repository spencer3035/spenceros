use crate::Direction;

use super::KeyCodeDef;
use quote::quote;
use std::collections::BTreeMap;

use proc_macro2::TokenStream;

pub fn generate_enum(def: &KeyCodeDef) -> TokenStream {
    let mut names: Vec<_> = def.key_codes.iter().map(|kc| &kc.name).collect();
    names.sort();
    names.dedup();
    quote! {
        #[derive(Debug)]
        enum KeyCode {
            #(#names,)*
        }
    }
}

pub fn generate_to_char(def: &KeyCodeDef) -> TokenStream {
    let names = def.key_codes.iter().map(|kc| &kc.name);
    let chars = def.key_codes.iter().map(|kc| &kc.char);
    quote! {
        fn to_char(&self) -> Option<char> {
            match self {
            #(
                KeyCode::#names => #chars,
            )*
                _ => None,
            }
        }
    }
}

/// Generates something like this
///
/// ```
/// struct KeyEvent {
///     is_down: bool,
///     code: KeyCode,
/// }
///
/// enum KeyCode {
///     Kc1,
///     Kc2,
///     Kc3,
///     Kc4,
/// }
///
/// fn get_event(code: u8, idx: u8) -> Option<KeyEvent> {
///     if false {
///         None
///     } else if idx == 0 {
///         if false {
///             None
///         } else if code == 0x01 {
///             Some(KeyEvent {
///                 code: KeyCode::Kc1,
///                 is_down: true,
///             })
///         } else if code == 0x02 {
///             Some(KeyEvent {
///                 code: KeyCode::Kc2,
///                 is_down: true,
///             })
///         } else {
///             None
///         }
///     } else if idx == 1 {
///         if false {
///             None
///         } else if code == 0x03 {
///             Some(KeyEvent {
///                 code: KeyCode::Kc3,
///                 is_down: true,
///             })
///         } else if code == 0x04 {
///             Some(KeyEvent {
///                 code: KeyCode::Kc4,
///                 is_down: true,
///             })
///         } else {
///             None
///         }
///     } else {
///         None
///     }
/// }
/// ```
pub fn generate_get_event(def: &KeyCodeDef) -> TokenStream {
    let mut impls: BTreeMap<u8, TokenStream> = BTreeMap::new();
    for kc in def.key_codes.iter() {
        let is_down = matches!(kc.dir, Direction::Down);
        let name = &kc.name;
        for (code_ii, code_curr) in kc.codes.codes.iter().enumerate() {
            let else_if = quote! {
                else if code == #code_curr {
                    Some(KeyEvent {
                        code : KeyCode::#name,
                        is_down : #is_down,
                    })
                }
            };
            if let Some(imp) = impls.get_mut(&(code_ii as u8)) {
                imp.extend(else_if);
            } else {
                let val = quote! {
                    if false {
                        None
                    }
                    #else_if
                };
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
        fn get_event(code: u8, idx : u8) -> Option<KeyEvent> {
            #blocks
        }
    }
}

/// Tries to generate something like:
/// ```
/// fn has_next(code: u8, index: u8) -> bool {
///     if false {
///         false
///     } else if index == 0 {
///         false || code == 0x01 || code == 0x02 || code == 0x03
///     } else if index == 1 {
///         false || code == 0x04 || code == 0x05 || code == 0x06
///     } else {
///         false
///     }
/// }
/// ```
pub fn generate_has_next(def: &KeyCodeDef) -> TokenStream {
    let mut impls: BTreeMap<u8, TokenStream> = BTreeMap::new();
    for kc in def.key_codes.iter() {
        for (code_ii, code_win) in kc.codes.codes.windows(2).enumerate() {
            let code_curr = &code_win[0];
            let _code_next = &code_win[1];
            let or_code = quote! {
                || code == #code_curr
            };
            if let Some(imp) = impls.get_mut(&(code_ii as u8)) {
                imp.extend(or_code);
            } else {
                let val = quote! {
                    false #or_code
                };
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
        pub fn has_next(code: u8, idx : u8) -> bool {
            #blocks
        }
    }
}
