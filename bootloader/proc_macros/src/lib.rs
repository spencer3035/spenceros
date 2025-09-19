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
/// #[derive(FromScancodes, Debug)]
/// enum KeyCode {
///    #[scan(down=[0x01],up=[0x81])]
///    KcEsc,
///    #[scan(down=[0x02],up=[0x82],ch='1')]
///    KcOne,
///    #[scan(down=[0xE0, 0x03],up=[0xE0, 0x83],ch='2')]
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
