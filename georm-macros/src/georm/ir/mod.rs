use quote::quote;

pub mod simple_relationship;
use simple_relationship::{OneToMany, OneToOne, SimpleRelationship};

pub mod m2m_relationship;
use m2m_relationship::M2MRelationship;

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(georm))]
pub struct GeormStructAttributes {
    pub table: String,
    #[deluxe(default = Vec::new())]
    pub one_to_one: Vec<SimpleRelationship<OneToOne>>,
    #[deluxe(default = Vec::new())]
    pub one_to_many: Vec<SimpleRelationship<OneToMany>>,
    #[deluxe(default = Vec::new())]
    pub many_to_many: Vec<M2MRelationship>,
}

#[derive(deluxe::ExtractAttributes, Clone)]
#[deluxe(attributes(georm))]
struct GeormFieldAttributes {
    #[deluxe(default = false)]
    pub id: bool,
    #[deluxe(default = None)]
    pub relation: Option<O2ORelationship>,
}

#[derive(deluxe::ParseMetaItem, Clone, Debug)]
pub struct O2ORelationship {
    pub entity: syn::Type,
    pub table: String,
    #[deluxe(default = String::from("id"))]
    pub remote_id: String,
    #[deluxe(default = false)]
    pub nullable: bool,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct GeormField {
    pub ident: syn::Ident,
    pub field: syn::Field,
    pub ty: syn::Type,
    pub id: bool,
    pub relation: Option<O2ORelationship>,
}

impl GeormField {
    pub fn new(field: &mut syn::Field) -> Self {
        let ident = field.clone().ident.unwrap();
        let ty = field.clone().ty;
        let attrs: GeormFieldAttributes =
            deluxe::extract_attributes(field).expect("Could not extract attributes from field");
        let GeormFieldAttributes { id, relation } = attrs;
        Self {
            ident,
            field: field.to_owned(),
            id,
            ty,
            relation,
        }
    }
}

impl From<&GeormField> for proc_macro2::TokenStream {
    fn from(value: &GeormField) -> Self {
        let Some(relation) = value.relation.clone() else {
            return quote! {};
        };
        let function = syn::Ident::new(
            &format!("get_{}", relation.name),
            proc_macro2::Span::call_site(),
        );
        let entity = &relation.entity;
        let return_type = if relation.nullable {
            quote! { Option<#entity> }
        } else {
            quote! { #entity }
        };
        let query = format!(
            "SELECT * FROM {} WHERE {} = $1",
            relation.table, relation.remote_id
        );
        let local_ident = &value.field.ident;
        let fetch = if relation.nullable {
            quote! { fetch_optional }
        } else {
            quote! { fetch_one }
        };
        quote! {
            pub async fn #function(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<#return_type> {
                ::sqlx::query_as!(#entity, #query, self.#local_ident).#fetch(pool).await
            }
        }
    }
}
