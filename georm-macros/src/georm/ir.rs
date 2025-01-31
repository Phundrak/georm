use quote::quote;

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(georm))]
pub struct GeormStructAttributes {
    pub table: String,
    #[deluxe(default = Vec::new())]
    pub one_to_many: Vec<O2MRelationship>,
    #[deluxe(default = Vec::new())]
    pub many_to_many: Vec<M2MRelationship>,
}

#[derive(deluxe::ParseMetaItem)]
pub struct O2MRelationship {
    pub name: String,
    pub remote_id: String,
    pub table: String,
    pub entity: syn::Type,
}

impl From<&O2MRelationship> for proc_macro2::TokenStream {
    fn from(value: &O2MRelationship) -> Self {
        let query = format!(
            "SELECT * FROM {} WHERE {} = $1",
            value.table, value.remote_id
        );
        let entity = &value.entity;
        let function = syn::Ident::new(
            &format!("get_{}", value.name),
            proc_macro2::Span::call_site(),
        );
        quote! {
            pub async fn #function(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<Vec<#entity>> {
                ::sqlx::query_as!(#entity, #query, self.get_id()).fetch_all(pool).await
            }
        }
    }
}

#[derive(deluxe::ParseMetaItem, Clone)]
pub struct M2MLink {
    pub table: String,
    pub from: String,
    pub to: String,
}

//#[georm(
//    table = "users",
//    many_to_many = [
//        {
//            name = friends,
//            entity: User,
//            link = { table = "user_friendships", from: "user1", to "user2" }
//        }
//    ]
//)]
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

#[derive(deluxe::ExtractAttributes, Clone)]
#[deluxe(attributes(georm))]
struct GeormFieldAttributes {
    #[deluxe(default = false)]
    pub id: bool,
    #[deluxe(default = None)]
    pub relation: Option<O2ORelationship>,
}

// #[georm(relation = { name = profile, id = "id", entity = Profile, nullable })]
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
