mod georm;
use georm::georm_derive_macro2;

/// Generates GEORM code for Sqlx for a struct.
///
/// # Panics
///
/// May panic if errors arise while parsing and generating code.
#[proc_macro_derive(Georm, attributes(georm))]
pub fn georm_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    georm_derive_macro2(item.into()).unwrap().into()
}
