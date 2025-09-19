use quote::quote;

use proc_macro2::{Literal, TokenStream};
use quote::ToTokens;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, Token,
};

mod gen;

struct KeyCodeDef {
    key_codes: Vec<KeyCodeItem>,
}

impl ToTokens for KeyCodeDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let enum_def = gen::generate_enum(self);
        let to_char = gen::generate_to_char(self);
        let has_next = gen::generate_has_next(self);
        let get_event = gen::generate_get_event(self);

        let q = quote! {
            #enum_def

            impl KeyCode {
                #to_char
                #has_next
                #get_event
            }
        };

        tokens.extend(q)
    }
}

struct ParenItem(pub KeyCodeItem);
impl Parse for ParenItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // `parenthesized!` creates a new sub‑stream that is bounded by '(' and ')'.
        let content;
        syn::parenthesized!(content in input);
        // Now parse a `MyItem` from that inner stream.
        let item = content.parse::<KeyCodeItem>()?;
        Ok(ParenItem(item))
    }
}

impl Parse for KeyCodeDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let punctuated: Punctuated<ParenItem, Token![,]> = Punctuated::parse_terminated(input)?;

        Ok(Self {
            key_codes: punctuated.into_iter().map(|ii| ii.0).collect(),
        })
    }
}

struct KeyCodeItem {
    codes: Codes,
    name: Ident,
    dir: Direction,
    char: Char,
}

impl Parse for KeyCodeItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let code;
        _ = bracketed!(code in input);
        let codes = code.parse()?;
        _ = input.parse::<Token![,]>();
        let name = input.parse()?;
        _ = input.parse::<Token![,]>();
        let dir = input.parse()?;
        _ = input.parse::<Token![,]>();
        let char = input.parse()?;
        Ok(Self {
            codes,
            name,
            dir,
            char,
        })
    }
}

struct Codes {
    codes: Vec<Literal>,
}

impl Parse for Codes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut codes = Vec::new();
        while let Ok(item) = input.parse() {
            codes.push(item);
            _ = input.parse::<Token![,]>();
        }

        Ok(Codes { codes })
    }
}

enum Direction {
    Up,
    Down,
}

impl Parse for Direction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let str = ident.to_string();
        if str == "Up" {
            Ok(Direction::Up)
        } else if str == "Down" {
            Ok(Direction::Down)
        } else {
            Err(input.error("Expected \"Up\" or \"Down\""))
        }
    }
}

enum Char {
    None,
    Some(Literal),
}

impl Parse for Char {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(lit) = input.parse::<Literal>() {
            Ok(Char::Some(lit))
        } else if let Ok(ident) = input.parse::<Ident>() {
            if ident == "None" {
                Ok(Char::None)
            } else {
                Err(input.error("Needs to be a char or 'None'"))
            }
        } else {
            Err(input.error("Needs to be a char or 'None'"))
        }
    }
}

impl ToTokens for Char {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let q = match self {
            Char::None => quote! {None},
            Char::Some(lit) => quote! {Some(#lit)},
        };

        tokens.extend(q);
    }
}

#[proc_macro]
pub fn define_keycodes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let def = parse_macro_input!(input as KeyCodeDef);
    quote! {#def}.into()
}
