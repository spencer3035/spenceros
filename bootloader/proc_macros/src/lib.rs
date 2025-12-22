use quote::quote;

use syn::DeriveInput;

mod derive;

/// Derives the FromScancode trait on an enum
///
/// The variants need to each have an annotation describing the scancode sequence for a down press
/// and an up press. It is also valid for a list to be empty `[]`, in which case it will be
/// impossible to generate that variant.
///
/// ```
/// pub trait FromScancodes: Sized {
///     /// If the current code and index is terminal, assuming that all previous vales were terminal
///     fn has_next(code: u8, index: u8) -> bool;
///     /// Gets the scancode given the terminal code and the number of codes, as well as if it is a
///     /// down press or not (_, true) is downpress, (_, false) is a release.
///     fn from_scancode_and_depth(code: u8, index: u8) -> Option<(Self, bool)>;
///     /// Tries to conver the key to an unshifted character
///     fn to_char_lower(&self) -> Option<char>;
///     /// Tries to conver the key to a shifted character
///     fn to_char_upper(&self) -> Option<char>;
/// }
///
/// use proc_macros::FromScancodes;
/// #[derive(FromScancodes, Debug)]
/// enum KeyCode {
///    #[scan(press=[0x01],release=[0x81])]
///    KcEsc,
///    #[scan(press=[0x02],release=[0x82],lower='1',upper='!')]
///    KcOne,
///    #[scan(press=[0xE0, 0x03],release=[0xE0, 0x83],lower='2',upper='@')]
///    KcTwo,
/// }
/// ```
#[proc_macro_derive(FromScancodes, attributes(scan))]
pub fn from_scancodes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match derive::from_scancodes_impl(&input) {
        Ok(output) => quote! {#output}.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
