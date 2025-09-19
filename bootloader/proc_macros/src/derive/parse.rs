use proc_macro2::Literal;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::DataEnum;
use syn::DeriveInput;
use syn::Error;
use syn::Ident;
use syn::Token;

use super::SCAN_TRAIT;

/// Information about the variant along with it's identifier
pub struct VariantInfo {
    pub name: Ident,
    pub down: Vec<Literal>,
    pub up: Vec<Literal>,
    pub ch: Option<Literal>,
}

/// Expect the attribute to be applied to an enum
pub fn expect_enum(input: &DeriveInput) -> syn::Result<&DataEnum> {
    match &input.data {
        syn::Data::Enum(e) => Ok(e),
        _ => {
            let msg = format!("{SCAN_TRAIT} can only be derived for enums");
            Err(Error::new_spanned(&input.ident, msg))
        }
    }
}

/// Values that are contained in each attribute
pub struct ScanValues {
    pub down: Vec<Literal>,
    pub up: Vec<Literal>,
    pub ch: Option<Literal>,
}

impl Parse for ScanValues {
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

        Ok(ScanValues { down, up, ch })
    }
}

// Helper struct to parse `ident = value`
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

/// Helper struct to parse a bracketed list like `[a,b,c]`
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
