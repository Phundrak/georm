use super::ir::GeormField;
use quote::quote;

#[derive(Debug)]
pub enum IdType {
    Simple {
        field_name: syn::Ident,
        field_type: syn::Type,
    },
    Composite {
        fields: Vec<IdField>,
        field_type: syn::Ident,
    },
}

#[derive(Debug, Clone)]
pub struct IdField {
    pub name: syn::Ident,
    pub ty: syn::Type,
}

fn field_to_code(field: &GeormField) -> proc_macro2::TokenStream {
    let ident = field.ident.clone();
    let ty = field.ty.clone();
    quote! {
        pub #ident: #ty
    }
}

fn generate_struct(
    ast: &syn::DeriveInput,
    fields: &[GeormField],
) -> (syn::Ident, proc_macro2::TokenStream) {
    let struct_name = &ast.ident;
    let id_struct_name = quote::format_ident!("{struct_name}Id");
    let vis = &ast.vis;
    let fields: Vec<proc_macro2::TokenStream> = fields
        .iter()
        .filter_map(|field| {
            if field.id {
                Some(field_to_code(field))
            } else {
                None
            }
        })
        .collect();
    let code = quote! {
        #vis struct #id_struct_name {
            #(#fields),*
        }
    };
    (id_struct_name, code)
}

pub fn create_primary_key(
    ast: &syn::DeriveInput,
    fields: &[GeormField],
) -> (IdType, proc_macro2::TokenStream) {
    let georm_id_fields: Vec<&GeormField> = fields.iter().filter(|field| field.id).collect();
    let id_fields: Vec<IdField> = georm_id_fields
        .iter()
        .map(|field| IdField {
            name: field.ident.clone(),
            ty: field.ty.clone(),
        })
        .collect();
    match id_fields.len() {
        0 => panic!("No ID field found"),
        1 => (
            IdType::Simple {
                field_name: id_fields[0].name.clone(),
                field_type: id_fields[0].ty.clone(),
            },
            quote! {},
        ),
        _ => {
            let (struct_name, struct_code) = generate_struct(ast, fields);
            (
                IdType::Composite {
                    fields: id_fields.clone(),
                    field_type: struct_name,
                },
                struct_code,
            )
        }
    }
}
