use ir::GeormField;
use quote::quote;

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
    let trait_impl = trait_implementation::derive_trait(&ast, &struct_attrs.table, &fields, &id);
    let relationships = relationships::derive_relationships(&ast, &struct_attrs, &fields, &id);
    let code = quote! {
        #trait_impl
        #relationships
    };
    println!("{code}");
    Ok(code)
}
