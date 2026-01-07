use darling::{FromDeriveInput, FromVariant};
use quote::quote;
use syn::{GenericParam, parse_quote};

#[derive(FromVariant)]
#[darling(attributes(event))]
struct MyVariantInput {
    ident: syn::Ident,

    #[darling(default)]
    rename: Option<String>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(event), supports(enum_any))]
struct Input {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<MyVariantInput, ()>,

    #[darling(default)]
    prefix: Option<String>,
}

struct ExtendableGenerics {
    generics: syn::Generics,
}

impl From<syn::Generics> for ExtendableGenerics {
    fn from(generics: syn::Generics) -> Self {
        Self { generics }
    }
}

impl From<ExtendableGenerics> for syn::Generics {
    fn from(extended_generics: ExtendableGenerics) -> Self {
        extended_generics.generics
    }
}

impl ExtendableGenerics {
    fn add_param(&mut self, param: syn::GenericParam) {
        match param {
            GenericParam::Lifetime(lt) => self.add_lt(lt),
            GenericParam::Type(ty) => self.add_type(ty),
            GenericParam::Const(const_generic) => self.add_const(const_generic),
        }
    }

    fn add_lt(&mut self, lt: syn::LifetimeParam) {
        let place = self
            .generics
            .params
            .iter()
            .position(|v| matches!(v, syn::GenericParam::Type(_) | syn::GenericParam::Const(_)))
            .unwrap_or(self.generics.params.len());

        self.generics
            .params
            .insert(place, syn::GenericParam::Lifetime(lt));
    }

    fn add_const(&mut self, const_generic: syn::ConstParam) {
        self.generics
            .params
            .push(syn::GenericParam::Const(const_generic));
    }

    fn add_type(&mut self, ty: syn::TypeParam) {
        let place = self
            .generics
            .params
            .iter()
            .position(|v| matches!(v, syn::GenericParam::Const(_)))
            .unwrap_or(self.generics.params.len());

        self.generics
            .params
            .insert(place, syn::GenericParam::Type(ty));
    }
}

#[manyhow::manyhow(proc_macro_derive(Event, attributes(event)))]
pub fn my_macro(input: syn::DeriveInput) -> manyhow::Result<proc_macro2::TokenStream> {
    let Input {
        ident,
        generics,
        prefix,
        data,
    } = Input::from_derive_input(&input)?;

    // ensured by supports(enum_any)
    #[expect(
        clippy::unwrap_used,
        reason = "this is ensured by supports(enum_any) in the darling derive."
    )]
    let enum_data = data.take_enum().unwrap();

    let mut publishable_impl_generics: ExtendableGenerics = generics.clone().into();

    publishable_impl_generics.add_param(parse_quote!(__macro_generic_T: 'static));
    publishable_impl_generics.add_param(parse_quote!('__macro_generic_frame));
    publishable_impl_generics.add_param(parse_quote!('__macro_generic_bp));

    let (impl_generics_publishable, _, where_clause_publishable) =
        publishable_impl_generics.generics.split_for_impl();
    let mut as_typed_event_impl_generics: ExtendableGenerics = generics.clone().into();

    as_typed_event_impl_generics.add_param(parse_quote!('__macro_generic_a));

    let (impl_generics_as_typed_event, _, where_clause_as_typed_event) =
        as_typed_event_impl_generics.generics.split_for_impl();
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let ident_fn = enum_data.iter().map(|v| {
        let variant_ident = &v.ident;

        let variant_ident_str = syn::LitStr::new(
            &v.rename
                .clone()
                .unwrap_or_else(|| prefix.clone().unwrap_or_default() + &variant_ident.to_string()),
            variant_ident.span(),
        );

        quote! {
            #ident::#variant_ident { .. } => #variant_ident_str
        }
    });

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics_publishable ::library_package::traits::Publishable<#ident #type_generics> for ::anathema::component::Context<'__macro_generic_frame,'__macro_generic_bp, __macro_generic_T> #where_clause_publishable {
            fn publish_typed(&mut self, event: #ident #type_generics) {
                self.publish(::library_package::traits::Event::event_ident(&event), event)
            }
        }


        #[automatically_derived]
        impl #impl_generics_as_typed_event ::library_package::traits::AsTypedEvent<#ident #type_generics> for ::anathema::component::UserEvent<'__macro_generic_a>  #where_clause_as_typed_event {
            fn as_typed_event(&self) -> Option<&#ident #type_generics> {
                let name = self.name();
                let data: &#ident #type_generics = self.data_checked()?;

                bool::then_some(::library_package::traits::Event::event_ident(data) == name, data)
            }
        }


        impl #impl_generics ::library_package::traits::Event for #ident #type_generics #where_clause {
            fn event_ident(&self) -> &'static str {
                match self {
                    #(#ident_fn),*,
                }
            }
        }
    })
}
