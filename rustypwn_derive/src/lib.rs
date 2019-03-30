#![feature(proc_macro_diagnostic, box_syntax, fmt_internals)]
#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Data, Meta, Lit, DataEnum, Variant, MetaNameValue};

/// implements derive macro ActionArg on an action argument type
///
/// # Example
///
/// Send within action is like
///
/// ```ignore
/// pub enum Action {
///     Send {
///         timeout: Timeout,
///         content: Vec<u8>
///     },
///     // ... other actions
/// }
/// ```
///
/// With the macro, it should be like:
///
/// ```ignore
/// #[derive(ActionArg)]
/// pub enum Action {
///     Send {
///         #[default = "0x1000"]
///         timeout: Timeout,
///         #[default = "b\"\".to_vec()"]
///         content: Vec<u8>,
///     }
/// }
/// ```
///
/// And the goal of `ActionArg` derive macro is to implement: 
///
/// ```ignore
/// pub struct Send {
///     pub timeout: Timeout,
///     pub content: Vec<u8>,
/// }
///
/// impl TryFrom<Action> for Send {
///     type Error = super::error::Error;
///
///     fn try_from(action: Action) -> Result<Self, Self::Error> {
///         match action {
///             Action::Send {
///                 timeout,
///                 content,
///             } => {
///                 Ok(Self {
///                     timeout: timeout,
///                     content: content
///                 })
///             },
///             _ => Err(Self::Error{ kind: super::error::ErrorKind::IncorrectAction} ),
///         }
///     }
/// }
///
/// impl Default for Send {
///     fn default() -> Self {
///         Self {
///             timeout: 0x1000, // default value
///             content: b"".to_vec(), // default value
///         }
///     }
/// }
///
/// impl Send {
///     pub fn new() -> Self {
///         Self::default()
///     }
///
///     // builder pattern to set all fields
///     pub fn timeout(mut self, timeout: Timeout) -> Self {
///         self.timeout = timeout;
///         self
///     }
///
///     pub fn content(mut self, content: Vec<u8>) -> Self {
///         self.content = content;
///         self
///     }
/// }
/// ```
#[proc_macro_derive(ActionArg, attributes(default))]
pub fn arction_arg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let data = match input.data {
        Data::Enum(data) => {
            data
        },
        _ => {
            input
                .span()
                .unwrap()
                .error("ActionArg cannot be used on item other than enum")
                .emit();

            panic!("ActionArg cannot be used on item other than enum")
        }
    };

    let variants_impl = impl_variants(&data);

    let output = quote! {
        use std::convert::TryFrom;

        #(#variants_impl)*
    };

    proc_macro::TokenStream::from(output)
}

fn impl_variants(data: &DataEnum) -> Vec<TokenStream> {
    let mut impls = Vec::new();
    for variant in data.variants.iter() {
        impls.push(impl_single_variant(&variant));
    }
    impls
}

fn impl_single_variant(variant: &Variant) -> TokenStream {
    let name = &variant.ident;

    let mut fields = Vec::new();
    let mut fields_default = Vec::new();
    let mut fields_methods = Vec::new();
    let mut fields_names = Vec::new();
    let mut fields_setups = Vec::new();

    for field in variant.fields.iter() {
        let field_name = match &field.ident {
            Some(ident) => ident,
            None => {
                field.span()
                    .unwrap()
                    .error("ActionArg all enum variants' fields must be named")
                    .emit();

                panic!("incorrect enum")
            }
        };
        fields_names.push(field_name);
        fields_setups.push(quote! {
            #field_name: #field_name,
        });

        let ty = &field.ty;
        fields.push(quote! {
            pub #field_name: #ty,
        });

        let default_attr = &field.attrs[0];
        let default_val = match default_attr.parse_meta().unwrap() {
            Meta::NameValue(MetaNameValue { lit: Lit::Str(lit_str), .. }) => {
                lit_str.parse::<TokenStream>().unwrap()
            },
            _ => {
                default_attr.span()
                    .unwrap()
                    .error("field default should be formed as #[default = \"VALUE\"]")
                    .emit();

                panic!("incorrect field default value")
            }
        };

        fields_default.push(quote! {
            #field_name: #default_val,
        });


        fields_methods.push(quote! {
            pub fn #field_name(mut self, #field_name: #ty) -> Self {
                self.#field_name = #field_name;
                self
            }
        });
    }

    let try_from_impl = quote! {
        impl TryFrom<Action> for #name {
            type Error = super::error::Error;

            fn try_from(action: Action) -> Result<Self, Self::Error> {
                match action {
                    Action::#name {
                        #(#fields_names),*
                    } => {
                        Ok(Self {
                            #(#fields_setups)*
                        })
                    },

                    _ => Err(Self::Error{ kind: super::error::ErrorKind::IncorrectAction }),
                }
            }
        }
    };

    quote! {
        pub struct #name {
            #(#fields)*
        }

        impl Default for #name {
            fn default() -> Self {
                Self {
                    #(#fields_default)*
                }
            }
        }

        #try_from_impl

        impl #name {
            pub fn new() -> Self {
                Self::default()
            }

            #(#fields_methods)*
        }
    }
}
