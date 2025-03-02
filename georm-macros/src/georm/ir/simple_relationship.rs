use quote::quote;

pub trait SimpleRelationshipType {}

#[derive(deluxe::ParseMetaItem, Default)]
pub struct OneToOne;
impl SimpleRelationshipType for OneToOne {}

#[derive(deluxe::ParseMetaItem, Default)]
pub struct OneToMany;
impl SimpleRelationshipType for OneToMany {}

#[derive(deluxe::ParseMetaItem)]
pub struct SimpleRelationship<T>
where
    T: SimpleRelationshipType + deluxe::ParseMetaItem + Default,
{
    pub name: String,
    pub remote_id: String,
    pub table: String,
    pub entity: syn::Type,
    #[deluxe(default = T::default())]
    _phantom: T,
}

impl<T> SimpleRelationship<T>
where
    T: SimpleRelationshipType + deluxe::ParseMetaItem + Default,
{
    pub fn make_query(&self) -> String {
        format!("SELECT * FROM {} WHERE {} = $1", self.table, self.remote_id)
    }

    pub fn make_function_name(&self) -> syn::Ident {
        syn::Ident::new(
            &format!("get_{}", self.name),
            proc_macro2::Span::call_site(),
        )
    }
}

impl From<&SimpleRelationship<OneToOne>> for proc_macro2::TokenStream {
    fn from(value: &SimpleRelationship<OneToOne>) -> Self {
        let query = value.make_query();
        let entity = &value.entity;
        let function = value.make_function_name();
        quote! {
            pub async fn #function(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<Option<#entity>> {
                ::sqlx::query_as!(#entity, #query, self.get_id()).fetch_optional(pool).await
            }
        }
    }
}

impl From<&SimpleRelationship<OneToMany>> for proc_macro2::TokenStream {
    fn from(value: &SimpleRelationship<OneToMany>) -> Self {
        let query = value.make_query();
        let entity = &value.entity;
        let function = value.make_function_name();
        quote! {
            pub async fn #function(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<Vec<#entity>> {
                ::sqlx::query_as!(#entity, #query, self.get_id()).fetch_all(pool).await
            }
        }
    }
}
