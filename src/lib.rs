use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Fields};

fn nice_enum_impl(input: DeriveInput) -> TokenStream {
    let input_variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("NiceEnum can only be derived for enums"),
    };

    let vis = input.vis;
    let source_ident = input.ident.clone();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let kind_ident_name = format!("{}Kind", input.ident);
    let kind_ident = Ident::new(&kind_ident_name, Span::call_site());

    struct Variant {
        ident: syn::Ident,
        qualified: TokenStream,
        as_method: Option<TokenStream>,
        as_mut_method: Option<TokenStream>,
        into_method: Option<TokenStream>,
        is_variant_method: syn::Ident,
        source_arm: TokenStream,
    }

    let variants: Vec<Variant> = input_variants
        .into_iter()
        .map(|variant| {
            let mut ident = variant.ident.clone();
            ident.set_span(Span::call_site());

            let ident_str = ident.clone().to_string();
            let ident_snake_case = ident_str.to_case(Case::Snake);

            let qualified = quote! { #kind_ident::#ident };

            let is_variant_method = format!("is_{}", &ident_snake_case);
            let is_variant_method = Ident::new(&is_variant_method, Span::call_site());

            let source_ident = quote! { Self::#ident };

            let (source_arm, as_method, as_mut_method, into_method) = match &variant.fields {
                Fields::Named(_) => (quote! { #source_ident { .. } }, None, None, None),
                Fields::Unnamed(fields) => {
                    if fields.unnamed.len() == 1 {
                        // SAFETY: We know that there is exactly one field in the variant.
                        let inner = fields.unnamed.first().unwrap();
                        let inner_ty = &inner.ty;

                        let as_method = format!("as_{}", &ident_snake_case);
                        let as_method = Ident::new(&as_method, Span::call_site());

                        let as_method = quote! {
                            #vis fn #as_method(&self) -> Option<&#inner_ty> {
                                match self {
                                    #source_ident(v) => Some(v),
                                    _ => None,
                                }
                            }
                        };

                        let as_mut_method = format!("as_{}_mut", &ident_snake_case);
                        let as_mut_method = Ident::new(&as_mut_method, Span::call_site());

                        let as_mut_method = quote! {
                            #vis fn #as_mut_method(&mut self) -> Option<&mut #inner_ty> {
                                match self {
                                    #source_ident(v) => Some(v),
                                    _ => None,
                                }
                            }
                        };

                        let into_method = format!("into_{}", &ident_snake_case);
                        let into_method = Ident::new(&into_method, Span::call_site());

                        let into_method = quote! {
                            #vis fn #into_method(self) -> Option<#inner_ty> {
                                match self {
                                    #source_ident(v) => Some(v),
                                    _ => None,
                                }
                            }
                        };

                        (
                            quote! { #source_ident(_) },
                            Some(as_method),
                            Some(as_mut_method),
                            Some(into_method),
                        )
                    } else {
                        (quote! { #source_ident(_) }, None, None, None)
                    }
                }
                Fields::Unit => (quote! { #source_ident }, None, None, None),
            };

            Variant {
                ident,
                qualified,
                as_method,
                as_mut_method,
                into_method,
                is_variant_method,
                source_arm,
            }
        })
        .collect();

    let enum_kind_body: TokenStream = variants
        .iter()
        .map(|variant| {
            let name = &variant.ident;
            quote! { #name, }
        })
        .collect();

    let enum_kind_impl = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #vis enum #kind_ident {
            #enum_kind_body
        }
    };

    let source_kind_body: TokenStream = variants
        .iter()
        .map(|variant| {
            let arm = &variant.source_arm;
            let arm_expression = &variant.qualified;
            quote!(#arm => #arm_expression,)
        })
        .collect();

    let source_kind_fn = quote! {
        #vis fn kind(&self) -> #kind_ident {
            match self {
                #source_kind_body
            }
        }
    };

    let source_is_variant_fn: TokenStream = variants
        .iter()
        .map(|variant| {
            let method = &variant.is_variant_method;
            let qualified = &variant.qualified;

            quote! {
                #vis fn #method(&self) -> bool {
                    matches!(self.kind(), #qualified)
                }
            }
        })
        .collect();

    let source_as_variant_fn: TokenStream = variants
        .iter()
        .filter_map(|variant| variant.as_method.clone())
        .collect();

    let source_as_mut_variant_fn: TokenStream = variants
        .iter()
        .filter_map(|variant| variant.as_mut_method.clone())
        .collect();


    let source_into_variant_fn: TokenStream = variants
        .iter()
        .filter_map(|variant| variant.into_method.clone())
        .collect();

    quote! {
        #enum_kind_impl

        impl #impl_generics #source_ident #ty_generics #where_clause {
            #source_kind_fn

            #source_is_variant_fn

            #source_as_variant_fn

            #source_as_mut_variant_fn

            #source_into_variant_fn
        }
    }
}

#[proc_macro_derive(NiceEnum)]
pub fn derive_struct_info(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    nice_enum_impl(input).into()
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_derives_expected() {
        let ast: DeriveInput = parse_quote! {
            pub enum MyEnum {
                Unit,
                NamedFields { a: u32 },
                UnnamedFields(u32),
            }
        };

        let actual_tokens = nice_enum_impl(ast);

        let expected_tokens = quote! {
            #[derive(Debug , Clone , Copy , PartialEq , Eq , PartialOrd , Ord , Hash)]
            pub enum MyEnumKind {
                Unit,
                NamedFields,
                UnnamedFields,
            }

            impl MyEnum {
                pub fn kind(&self) -> MyEnumKind {
                    match self {
                        Self::Unit => MyEnumKind::Unit,
                        Self::NamedFields { .. } => MyEnumKind::NamedFields,
                        Self::UnnamedFields(_) => MyEnumKind::UnnamedFields,
                    }
                }

                pub fn is_unit(&self) -> bool {
                    matches!(self.kind(), MyEnumKind::Unit)
                }

                pub fn is_named_fields(&self) -> bool {
                    matches!(self.kind(), MyEnumKind::NamedFields)
                }

                pub fn is_unnamed_fields(&self) -> bool {
                    matches!(self.kind(), MyEnumKind::UnnamedFields)
                }

                pub fn as_unnamed_fields(&self) -> Option<&u32> {
                    match self {
                        Self::UnnamedFields(v) => Some(v),
                        _ => None,
                    }
                }

                pub fn as_unnamed_fields_mut(&mut self) -> Option<&mut u32> {
                    match self {
                        Self::UnnamedFields(v) => Some(v),
                        _ => None,
                    }
                }

                pub fn into_unnamed_fields(self) -> Option<u32> {
                    match self {
                        Self::UnnamedFields(v) => Some(v),
                        _ => None,
                    }
                }
            }
        };

        assert_eq!(actual_tokens.to_string(), expected_tokens.to_string());
    }
}
