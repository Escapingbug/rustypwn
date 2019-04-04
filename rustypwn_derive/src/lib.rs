#![feature(proc_macro_diagnostic, box_syntax, fmt_internals)]
#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DataEnum, DeriveInput, Ident, Lit, Meta, MetaNameValue, Variant,
    parse::Parse, parse::ParseStream,
};

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
/// impl From<Send> for Action {
///     fn from(action: Send) -> Self {
///         Action::Send {
///             timeout: action.timeout,
///             content: action.content
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
///
/// pub fn send() -> Send {
///     Send::default()
/// }
/// ```
#[proc_macro_derive(ActionArg, attributes(default))]
pub fn arction_arg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let data = match input.data {
        Data::Enum(data) => data,
        _ => {
            input
                .span()
                .unwrap()
                .error("ActionArg cannot be used on item other than enum")
                .emit();

            panic!("ActionArg cannot be used on item other than enum")
        }
    };

    let variants_impl = impl_variants(&input.ident, &data);

    let output = quote! {
        use std::convert::TryFrom;

        #(#variants_impl)*
    };

    proc_macro::TokenStream::from(output)
}

fn impl_variants(enum_name: &Ident, data: &DataEnum) -> Vec<TokenStream> {
    let mut impls = Vec::new();
    for variant in data.variants.iter() {
        impls.push(impl_single_variant(enum_name, &variant));
    }
    impls
}

fn impl_single_variant(enum_name: &Ident, variant: &Variant) -> TokenStream {
    let name = &variant.ident;

    let mut fields = Vec::new();
    let mut fields_default = Vec::new();
    let mut fields_methods = Vec::new();
    let mut fields_setups = Vec::new();

    for field in variant.fields.iter() {
        let field_name = match &field.ident {
            Some(ident) => ident,
            None => {
                field
                    .span()
                    .unwrap()
                    .error("ActionArg all enum variants' fields must be named")
                    .emit();

                panic!("incorrect enum")
            }
        };
        fields_setups.push(quote! {
            #field_name: action.#field_name,
        });

        let ty = &field.ty;
        fields.push(quote! {
            pub #field_name: #ty,
        });

        let default_attr = &field.attrs[0];
        let default_val = match default_attr.parse_meta().unwrap() {
            Meta::NameValue(MetaNameValue {
                lit: Lit::Str(lit_str),
                ..
            }) => lit_str.parse::<TokenStream>().unwrap(),
            _ => {
                default_attr
                    .span()
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

    let from_impl = quote! {
        impl From<#name> for #enum_name {
            fn from(action: #name) -> Self {
                #enum_name::#name {
                    #(#fields_setups)*
                }
            }
        }
    };

    let name_lower = Ident::new(&name.to_string().to_ascii_lowercase(), Span::call_site());

    let helper_func_impl = quote! {
        pub fn #name_lower() -> #name {
            #name::default()
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

        #from_impl

        impl #name {
            pub fn new() -> Self {
                Self::default()
            }

            #(#fields_methods)*
        }


        #helper_func_impl
    }
}

struct CommaSepIdents {
    pub idents: syn::punctuated::Punctuated<Ident, syn::token::Comma>,
}

impl Parse for CommaSepIdents {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let parsed = syn::punctuated::Punctuated::<Ident, syn::token::Comma>::parse_terminated(input)?;
        Ok(Self {
            idents: parsed
        })
    }
}

/// helper attribute for action function
/// 
/// # Example
///
/// ```ignored
/// impl TubeInternal for Process {
/// //...
/// #[action(timeout, content)]
/// fn sendline(&mut self, action: Action) -> Result<(), Error> {
///     // code
/// }
/// //...
/// }
/// ```
///
/// becomes
///
///```ignored
/// impl TubeInternal for Process {
/// //...
/// fn sendline(&mut self, action: Action) -> Result<(), Error> {
///     match action {
///         Action::Sendline {
///             timeout,
///             content
///         } => {
///             // code
///         },
///         _ => panic!("incorrect action, internal bug"),
///     }
/// }
/// //...
/// }
/// ```
#[proc_macro_attribute]
pub fn action(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::ItemFn);
    let args = parse_macro_input!(attr as CommaSepIdents).idents;

    let vis = input.vis;
    let ident = input.ident;
    let mut matching = ident.to_string().chars().collect::<Vec<_>>();
    matching[0] = matching[0].to_uppercase().collect::<Vec<_>>()[0];
    let matching = Ident::new(&matching.iter().collect::<String>(), Span::call_site());
    let decl = input.decl.inputs;
    let ret = input.decl.output;
    let action_arg_name = match decl.iter().collect::<Vec<_>>()[1] {
        syn::FnArg::Captured(captured) => &captured.pat,
        _ => panic!("unable to get action arg"),
    };
    let block = input.block;
    let generics = input.decl.generics;
    let variadic = input.decl.variadic;

    let gen = quote! {
        #vis fn #ident #generics(#decl #variadic) #ret {
            match #action_arg_name {
                Action::#matching {
                    #(#args),*
                } => {
                    #block
                },
                _ => panic!("incorrect action, internal bug"),
            }
        }
    };
    proc_macro::TokenStream::from(gen)
}
