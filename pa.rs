//! Convert numbers to enum variants.
//!
//! This crate provides a procedural macro (`derive(N)`) to generate a method (`n`) for converting a primitive integer into the corresponding variant of an enum.
//!
//! # Example
//!
//! ```
//! use enumn::N;
//!
//! #[derive(PartialEq, Debug, N)]
//! enum Status {
//!     Success,
//!     Failure,
//! }
//!
//! fn main() {
//!     let s = Status::n(0);
//!     assert_eq!(s, Some(Status::Success));
//!
//!     let s = Status::n(1);
//!     assert_eq!(s, Some(Status::Failure));
//!
//!     let s = Status::n(2);
//!     assert_eq!(s, None);
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/enumn/0.1.13")]
#![allow(
    clippy::missing_panics_doc,
    clippy::needless_doctest_main,
    clippy::single_match_else
)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, Ident};

#[proc_macro_derive(N)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let variants = match input.data {
        Data::Enum(data) => data.variants,
        Data::Struct(_) | Data::Union(_) => panic!("input must be an enum"),
    };

    for variant in &variants {
        match variant.fields {
            Fields::Unit => {}
            Fields::Named(_) | Fields::Unnamed(_) => {
                let span = variant.ident.span();
                let err = Error::new(span, "enumn: variant with data is not supported");
                return err.to_compile_error().into();
            }
        }
    }

    let repr = match input.attrs.iter().find(|attr| attr.path.is_ident("repr")) {
        Some(attr) => {
            if let Ok(name) = attr.parse_args::<Ident>() {
                match name.to_string().as_str() {
                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32"
                    | "i64" | "i128" | "isize" => Some(quote!(#name)),
                    _ => None,
                }
            } else {
                None
            }
        }
        None => Some(quote!(i64)),
    };

    let signature = match &repr {
        Some(repr) => quote! { fn n(value: #repr) },
        None => quote! { fn n<REPR: Into<i64>>(value: REPR) },
    };

    let value = match &repr {
        Some(_) => quote!(value),
        None => quote! { <REPR as Into<i64>>::into(value) },
    };

    let ident = &input.ident;
    let declare_discriminants = variants.iter().map(|variant| {
        let variant = &variant.ident;
        quote! { const #variant: #repr = #ident::#variant as #repr; }
    });
    let match_discriminants = variants.iter().map(|variant| {
        let variant = &variant.ident;
        quote! { discriminant::#variant => Some(#ident::#variant), }
    });

    TokenStream::from(quote! {
        impl #ident {
            pub #signature -> Option<Self> {
                #[allow(non_camel_case_types)]
                struct discriminant;
                #[allow(non_upper_case_globals)]
                impl discriminant {
                    #(#declare_discriminants)*
                }
                match #value {
                    #(#match_discriminants)*
                    _ => None,
                }
            }
        }
    })
}
