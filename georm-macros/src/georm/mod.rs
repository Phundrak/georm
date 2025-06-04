use ir::GeormField;
use quote::quote;

mod defaultable_struct;
mod ir;
mod relationships;
mod trait_implementation;

fn extract_georm_field_attrs(
    ast: &mut syn::DeriveInput,
) -> deluxe::Result<(Vec<GeormField>, GeormField)> {
    let syn::Data::Struct(s) = &mut ast.data else {
        return Err(syn::Error::new_spanned(
            ast,
            "Cannot apply to something other than a struct",
        ));
    };
    let fields = s
        .fields
        .clone()
        .into_iter()
        .map(|mut field| GeormField::new(&mut field))
        .collect::<Vec<GeormField>>();
    let identifiers: Vec<GeormField> = fields
        .clone()
        .into_iter()
        .filter(|field| field.id)
        .collect();
    match identifiers.len() {
        0 => Err(syn::Error::new_spanned(
            ast,
            "Struct {name} must have one identifier",
        )),
        1 => Ok((fields, identifiers.first().unwrap().clone())),
        _ => {
            let id1 = identifiers.first().unwrap();
            let id2 = identifiers.get(1).unwrap();
            Err(syn::Error::new_spanned(id2.field.clone(), format!(
                "Field {} cannot be an identifier, {} already is one.\nOnly one identifier is supported.",
                id1.ident, id2.ident
            )))
        }
    }
}

pub fn georm_derive_macro2(
    item: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut ast: syn::DeriveInput = syn::parse2(item).expect("Failed to parse input");
    let struct_attrs: ir::GeormStructAttributes =
        deluxe::extract_attributes(&mut ast).expect("Could not extract attributes from struct");
    let (fields, id) = extract_georm_field_attrs(&mut ast)?;
    let relationships = relationships::derive_relationships(&ast, &struct_attrs, &fields, &id);
    let trait_impl = trait_implementation::derive_trait(&ast, &struct_attrs.table, &fields, &id);
    let defaultable_struct =
        defaultable_struct::derive_defaultable_struct(&ast, &struct_attrs, &fields);
    let from_row_impl = generate_from_row_impl(&ast, &fields);
    let code = quote! {
        #relationships
        #trait_impl
        #defaultable_struct
        #from_row_impl
    };
    Ok(code)
}

fn generate_from_row_impl(
    ast: &syn::DeriveInput,
    fields: &[GeormField],
) -> proc_macro2::TokenStream {
    let struct_name = &ast.ident;
    let field_idents: Vec<&syn::Ident> = fields.iter().map(|f| &f.ident).collect();
    let field_names: Vec<String> = fields.iter().map(|f| f.ident.to_string()).collect();

    quote! {
        impl<'r> ::sqlx::FromRow<'r, ::sqlx::postgres::PgRow> for #struct_name {
            fn from_row(row: &'r ::sqlx::postgres::PgRow) -> ::sqlx::Result<Self> {
                use ::sqlx::Row;
                Ok(Self {
                    #(#field_idents: row.try_get(#field_names)?),*
                })
            }
        }
    }
}
