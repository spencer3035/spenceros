use syn::punctuated::Punctuated;
use syn::Error;

use syn::DataEnum;

use syn::DeriveInput;

use syn::parse::ParseStream;

use syn::parse::Parse;

use proc_macro2::Literal;

use syn::Ident;
use syn::Token;

pub(crate) struct ScancodeItem {
    pub(crate) name: Ident,
    pub(crate) down: Vec<Literal>,
    pub(crate) up: Vec<Literal>,
    pub(crate) ch: Option<Literal>,
}

pub(crate) struct KeyValue<T: Parse> {
    pub(crate) key: Ident,
    pub(crate) _eq: Token![=],
    pub(crate) value: T,
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

pub(crate) struct ScanKeyValues {
    pub(crate) down: Vec<Literal>,
    pub(crate) up: Vec<Literal>,
    pub(crate) ch: Option<Literal>,
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

pub(crate) struct BracketedList<T> {
    pub(crate) _bracket: syn::token::Bracket,
    pub(crate) list: Punctuated<T, Token![,]>,
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

pub(crate) fn expect_enum(input: &DeriveInput) -> syn::Result<&DataEnum> {
    match &input.data {
        syn::Data::Enum(e) => Ok(e),
        _ => Err(Error::new_spanned(
            &input.ident,
            "FromScancodes can only be derived for enums",
        )),
    }
}
