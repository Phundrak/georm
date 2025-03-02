use quote::quote;

#[derive(deluxe::ParseMetaItem, Clone)]
pub struct M2MLink {
    pub table: String,
    pub from: String,
    pub to: String,
}

#[derive(deluxe::ParseMetaItem)]
pub struct M2MRelationship {
    pub name: String,
    pub entity: syn::Type,
    pub table: String,
    #[deluxe(default = String::from("id"))]
    pub remote_id: String,
    pub link: M2MLink,
}

pub struct Identifier {
    pub table: String,
    pub id: String,
}

pub struct M2MRelationshipComplete {
    pub name: String,
    pub entity: syn::Type,
    pub local: Identifier,
    pub remote: Identifier,
    pub link: M2MLink,
}

impl M2MRelationshipComplete {
    pub fn new(other: &M2MRelationship, local_table: &String, local_id: String) -> Self {
        Self {
            name: other.name.clone(),
            entity: other.entity.clone(),
            link: other.link.clone(),
            local: Identifier {
                table: local_table.to_string(),
                id: local_id,
            },
            remote: Identifier {
                table: other.table.clone(),
                id: other.remote_id.clone(),
            },
        }
    }
}

impl From<&M2MRelationshipComplete> for proc_macro2::TokenStream {
    fn from(value: &M2MRelationshipComplete) -> Self {
        let function = syn::Ident::new(
            &format!("get_{}", value.name),
            proc_macro2::Span::call_site(),
        );
        let entity = &value.entity;
        let query = format!(
            "SELECT remote.*
FROM {} local
JOIN {} link ON link.{} = local.{}
JOIN {} remote ON link.{} = remote.{}
WHERE local.{} = $1",
            value.local.table,
            value.link.table,
            value.link.from,
            value.local.id,
            value.remote.table,
            value.link.to,
            value.remote.id,
            value.local.id
        );
        quote! {
            pub async fn #function(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<Vec<#entity>> {
                ::sqlx::query_as!(#entity, #query, self.get_id()).fetch_all(pool).await
            }
        }
    }
}
