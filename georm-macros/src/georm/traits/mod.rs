use super::composite_keys::IdType;
use super::ir::GeormField;
use quote::quote;

mod create;
mod delete;
mod find;
mod update;
mod upsert;

fn generate_get_id(id: &IdType) -> proc_macro2::TokenStream {
    match id {
        IdType::Simple {
            field_name,
            field_type,
        } => {
            quote! {
                fn get_id(&self) -> #field_type {
                    self.#field_name.clone()
                }
            }
        }
        IdType::Composite { fields, field_type } => {
            let field_names: Vec<syn::Ident> = fields.iter().map(|f| f.name.clone()).collect();
            quote! {
                fn get_id(&self) -> #field_type {
                    #field_type {
                        #(#field_names: self.#field_names),*
                    }
                }
            }
        }
    }
}

pub fn derive_trait(
    ast: &syn::DeriveInput,
    table: &str,
    fields: &[GeormField],
    id: &IdType,
) -> proc_macro2::TokenStream {
    let ty = match id {
        IdType::Simple { field_type, .. } => quote! {#field_type},
        IdType::Composite { field_type, .. } => quote! {#field_type},
    };

    // define impl variables
    let ident = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    // generate
    let get_id = generate_get_id(id);
    let get_all = find::generate_find_all_query(table);
    let find_query = find::generate_find_query(table, id);
    let create_query = create::generate_create_query(table, fields);
    let update_query = update::generate_update_query(table, fields, id);
    let upsert_query = upsert::generate_upsert_query(table, fields, id);
    let delete_query = delete::generate_delete_query(table, id);
    quote! {
        impl #impl_generics Georm<#ty> for #ident #type_generics #where_clause {
            #get_all
            #get_id
            #find_query
            #create_query
            #update_query
            #upsert_query
            #delete_query
        }
    }
}
