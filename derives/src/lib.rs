mod listener;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn listener(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    let listener = listener::Builder {
        item, attrs,
    };

    match listener.extend() {
        Ok(item) => item.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
