use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeMap;
use syn::Ident;

use super::parse::VariantInfo;
use super::SCAN_TRAIT;

/// Generates an implementation for FromScancodes
pub fn gen_impl(items: &[VariantInfo], name: &Ident) -> TokenStream {
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

/// Generates a function like the following
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
fn gen_has_next(items: &[VariantInfo]) -> TokenStream {
    // Map from index to the block that will be placed inside the if statement at that index
    let mut impls: BTreeMap<u8, TokenStream> = BTreeMap::new();
    for kc in items.iter() {
        // Iterating over windows of size two ensures that the lower end of the window is not the
        // terminal value, so therefore it has a next value.
        for (code_ii, code_win) in kc.down.windows(2).enumerate() {
            let code_curr = &code_win[0];
            let or_code = quote! { || code == #code_curr };
            if let Some(imp) = impls.get_mut(&(code_ii as u8)) {
                // Add ` || code == 0x00`
                imp.extend(or_code);
            } else {
                // Put first `false` value to concatenate the rest of the values to
                let val = quote! { false #or_code };
                impls.insert(code_ii as u8, val);
            }
        }
        for (code_ii, code_win) in kc.up.windows(2).enumerate() {
            let code_curr = &code_win[0];
            let or_code = quote! { || code == #code_curr };
            if let Some(imp) = impls.get_mut(&(code_ii as u8)) {
                // Add ` || code == 0x00`
                imp.extend(or_code);
            } else {
                // Put first `false` value to concatenate the rest of the values to
                let val = quote! { false #or_code };
                impls.insert(code_ii as u8, val);
            }
        }
    }

    // Start the block chain to match the indices against
    let mut blocks = quote! {
        if false {
            false
        }
    };
    // Populate the block chain to match the indices against
    for (idx, imp) in impls.into_iter() {
        let block = quote! {
            else if idx == #idx  {
                #imp
            }
        };
        blocks.extend(block);
    }
    // End the block chain
    blocks.extend(quote! {
        else {
            false
        }
    });

    // Put it all together
    quote! {
        fn has_next(code: u8, idx : u8) -> bool {
            #blocks
        }
    }
}

/// Generates a function like the following
/// ```
/// #enum KeyCode {
/// #    KcEsc,
/// #    Kc1,
/// #    Kc2,
/// #    Kc3,
/// #    Kc4,
/// #}
/// #impl KeyCode {
/// #
/// fn to_char(&self) -> Option<char> {
///     match self {
///         Self::KcEsc => None,
///         Self::Kc1 => Some('1'),
///         Self::Kc2 => Some('2'),
///         Self::Kc3 => Some('3'),
///         Self::Kc4 => Some('4'),
///     }
/// }
/// #}
/// ```
fn gen_to_char(items: &[VariantInfo]) -> TokenStream {
    let names = items.iter().map(|i| &i.name);
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

/// Generates a function like the following
/// ```
/// # enum KeyCode {
/// #     Kc1,
/// #     Kc2,
/// #     Kc3,
/// #     Kc4,
/// # }
///
/// fn from_scancode_and_depth(code: u8, idx: u8) -> Option<(Self, bool)> {
///     if false {
///         None
///     } else if idx == 0 {
///         if false {
///             None
///         } else if code == 0x01 {
///             Some((Self::Kc1, true))
///         } else if code == 0x02 {
///             Some((Self::Kc2, true))
///         } else {
///             None
///         }
///     } else if idx == 1 {
///         if false {
///             None
///         } else if code == 0x03 {
///             Some((Self::Kc3, true))
///         } else if code == 0x04 {
///             Some((Self::Kc4, true))
///         } else {
///             None
///         }
///     } else {
///         None
///     }
/// }
/// ```
fn gen_from_scancode_and_depth(items: &[VariantInfo]) -> TokenStream {
    // Map from index to the block that will be placed inside the if statement at that index
    let mut impls: BTreeMap<u8, TokenStream> = BTreeMap::new();
    for kc in items.iter() {
        let name = &kc.name;
        // Down pressed
        for (code_ii, code_curr) in kc.down.iter().enumerate() {
            let else_if = quote! {
                else if code == #code_curr {
                    Some((Self::#name, true))
                }
            };
            if let Some(imp) = impls.get_mut(&(code_ii as u8)) {
                // Just put else if block in sequence
                imp.extend(else_if);
            } else {
                // Start chain with `if false { None } `
                let val = quote! { if false { None } #else_if };
                impls.insert(code_ii as u8, val);
            }
        }
        // Up presses
        for (code_ii, code_curr) in kc.up.iter().enumerate() {
            let else_if = quote! {
                else if code == #code_curr {
                    Some((Self::#name, false))
                }
            };
            if let Some(imp) = impls.get_mut(&(code_ii as u8)) {
                // Just put else if block in sequence
                imp.extend(else_if);
            } else {
                // Start chain with `if false { None } `
                let val = quote! { if false { None } #else_if };
                impls.insert(code_ii as u8, val);
            }
        }
    }

    // Put the trailing `else` statement on each inner block
    for (_key, imp) in impls.iter_mut() {
        imp.extend(quote! {
            else {
                None
            }
        });
    }

    // Put leading `if false` to start block chain for index matching
    let mut blocks = quote! {
        if false {
            None
        }
    };

    // Put all the cases for the indices
    for (idx, imp) in impls.into_iter() {
        let block = quote! {
            else if idx == #idx  {
                #imp
            }
        };
        blocks.extend(block);
    }

    // Put trailing block on index branches
    blocks.extend(quote! {
        else {
            None
        }
    });

    // Put it all together in the function
    quote! {
        fn from_scancode_and_depth(code: u8, idx : u8) -> Option<(Self, bool)> {
            #blocks
        }
    }
}
