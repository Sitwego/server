use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Fields, Ident, ItemStruct, LitStr, Token, parse_macro_input};

struct GetterArgs {
    name: Option<String>,
}

impl Parse for GetterArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        if !input.is_empty() {
            let ident: Ident = input.parse()?;
            if ident != "name" {
                return Err(syn::Error::new(ident.span(), "Expected 'name'"));
            }
            let _: Token![=] = input.parse()?;
            let lit: LitStr = input.parse()?;
            name = Some(lit.value());
        }
        Ok(GetterArgs { name })
    }
}

#[proc_macro_attribute]
pub fn impl_getter(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as GetterArgs);
    let getter_name = args.name.unwrap_or_else(|| "inner".to_string()); // Back to "inner"
    let getter_ident =
        syn::Ident::new(&getter_name, proc_macro2::Span::call_site());
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let (field_access, field_ty) = match &input.fields {
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() != 1 {
                return syn::Error::new(
                    fields.span(),
                    "Expected a tuple struct with exactly one field",
                )
                .to_compile_error()
                .into();
            }
            let field = &fields.unnamed[0];
            (quote! { &self.0 }, &field.ty) // Explicit & for clarity
        }
        Fields::Named(fields) => {
            if fields.named.len() != 1 {
                return syn::Error::new(
                    fields.span(),
                    "Expected a named struct with exactly one field",
                )
                .to_compile_error()
                .into();
            }
            let field = fields.named.first().unwrap();
            let field_name = field.ident.as_ref().unwrap();
            (quote! { &self.#field_name }, &field.ty) // Explicit & for clarity
        }
        Fields::Unit => {
            return syn::Error::new(
                input.span(),
                "Unit structs are not supported by impl_getter",
            )
            .to_compile_error()
            .into();
        }
    };

    let expanded = quote! {
        #input
        impl #name {
            pub fn #getter_ident(&self) -> &#field_ty {
                #field_access
            }
        }
    };

    TokenStream::from(expanded)
}
