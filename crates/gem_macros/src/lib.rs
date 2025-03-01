use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Ident, Meta};

#[proc_macro_derive(ChainAttributes, attributes(is_evm, is_stake, is_cosmos))]
pub fn chain_features_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let Data::Enum(data_enum) = input.data else {
        return syn::Error::new_spanned(name, "ChainAttributes can only be used on enums")
            .to_compile_error()
            .into();
    };

    let evm_variants: Vec<&Ident> = data_enum
        .variants
        .iter()
        .filter(|variant| variant.attrs.iter().any(|attr| has_attribute(attr, "is_evm")))
        .map(|v| &v.ident)
        .collect();

    let _stake_variants: Vec<&Ident> = data_enum
        .variants
        .iter()
        .filter(|variant| variant.attrs.iter().any(|attr| has_attribute(attr, "is_stake")))
        .map(|v| &v.ident)
        .collect();

    let _cosmos_variants: Vec<&Ident> = data_enum
        .variants
        .iter()
        .filter(|variant| variant.attrs.iter().any(|attr| has_attribute(attr, "is_cosmos")))
        .map(|v| &v.ident)
        .collect();

    let evm_enum = quote! {
        #[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, PartialEq)]
        #[typeshare(swift = "Equatable, CaseIterable, Sendable")]
        #[serde(rename_all = "lowercase")]
        #[strum(serialize_all = "lowercase")]
        pub enum EVMChain {
            #(#evm_variants),*
        }

        impl TryFrom<#name> for EVMChain {
            type Error = &'static str;

            fn try_from(chain: #name) -> Result<Self, Self::Error> {
                match chain {
                    #(#name::#evm_variants => Ok(EVMChain::#evm_variants),)*
                    _ => Err("Not an EVM chain"),
                }
            }
        }
    };

    let output = quote! {
        #evm_enum
    };
    output.into()
}

fn has_attribute(attr: &Attribute, name: &str) -> bool {
    if let Meta::Path(path) = &attr.meta {
        path.is_ident(name)
    } else {
        false
    }
}
