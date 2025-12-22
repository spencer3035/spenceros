use proc_macro2::TokenStream;
use syn::DeriveInput;

mod gen_code;
mod parse;

/// Attribute name we look for
const SCAN_ATTR: &str = "scan";
/// Trait name we implement
const SCAN_TRAIT: &str = "FromScancodes";

pub fn from_scancodes_impl(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let enum_data = parse::expect_enum(input)?;

    // Gather all the info into a handy list
    let mut items = Vec::new();
    for field in enum_data.variants.iter() {
        for attr in field.attrs.iter() {
            if attr.path().is_ident(SCAN_ATTR) {
                // Parse attribute
                let parse::ScanValues {
                    press,
                    release,
                    ch_lower,
                    ch_upper,
                } = attr.meta.require_list()?.parse_args()?;
                // Convert to helper struct
                let item = parse::VariantInfo {
                    name: field.ident.clone(),
                    press,
                    release,
                    ch_lower,
                    ch_upper,
                };

                items.push(item);
            }
        }
    }

    Ok(gen_code::gen_impl(&items, name))
}
