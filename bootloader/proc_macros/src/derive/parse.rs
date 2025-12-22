use proc_macro2::Literal;
use syn::DataEnum;
use syn::DeriveInput;
use syn::Error;
use syn::Ident;
use syn::Token;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;

use super::SCAN_TRAIT;

/// Information about the variant along with it's identifier
pub struct VariantInfo {
    pub name: Ident,
    pub press: Vec<Literal>,
    pub release: Vec<Literal>,
    pub ch_lower: Option<Literal>,
    pub ch_upper: Option<Literal>,
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
    pub press: Vec<Literal>,
    pub release: Vec<Literal>,
    pub ch_lower: Option<Literal>,
    pub ch_upper: Option<Literal>,
}

impl Parse for ScanValues {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Down
        let kv: KeyValue<BracketedList<Literal>> = input.parse()?;
        if kv.key != "press" {
            return Err(input.error("missing 'press'"));
        }
        let press = kv.value.list.into_iter().collect();
        _ = input.parse::<Token![,]>()?;
        // Up
        let kv: KeyValue<BracketedList<Literal>> = input.parse()?;
        if kv.key != "release" {
            return Err(input.error("missing 'release'"));
        }
        let release = kv.value.list.into_iter().collect();
        // Optional char
        let mut ch_lower = None;
        let mut ch_upper = None;
        if input.parse::<Token![,]>().is_ok() {
            let kv = input.parse::<KeyValue<Literal>>()?;
            if kv.key != "lower" {
                return Err(input.error("missing 'lower'"));
            } else {
                ch_lower = Some(kv.value)
            }
            if input.parse::<Token![,]>().is_ok() {
                let kv = input.parse::<KeyValue<Literal>>()?;
                if kv.key != "upper" {
                    return Err(input.error("missing 'upper'"));
                } else {
                    ch_upper = Some(kv.value)
                }
            }
        }

        Ok(ScanValues {
            press,
            release,
            ch_lower,
            ch_upper,
        })
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
