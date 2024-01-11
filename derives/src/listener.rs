use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Error, FnArg, Ident, ItemFn, Lit, LitBool, Token};

pub(crate) struct Builder {
    pub(crate) item: ItemFn,
    pub(crate) attrs: TokenStream,
}

impl Builder {
    pub(crate) fn extend(&self) -> Result<TokenStream, Error> {
        let signature = &self.item.sig;
        let mut attrs = self.attrs.clone().into_iter();
        let mut config = HashMap::new();

        if signature.asyncness.is_none() {
            return Err(Error::new_spanned(
                self.item.sig.fn_token,
                "expected `async` before function declaration",
            ));
        }

        while let Some(token) = attrs.next() {
            let token: TokenStream = token.into();

            if let Ok(_) = syn::parse::<Token![,]>(token.clone()) {
                continue;
            }

            if let Ok(token) = syn::parse::<Ident>(token) {
                let next = attrs.next();

                if next.is_none() {
                    return Err(Error::new_spanned(
                        token,
                        "expected `=` after attribute name",
                    ));
                }

                let next = attrs.next();

                if next.is_none() {
                    return Err(Error::new_spanned(token, "expected value after `=`"));
                }

                let next = syn::parse::<Lit>(next.unwrap().into());

                if next.is_err() {
                    return Err(Error::new_spanned(
                        token,
                        "expected literal value after `=`",
                    ));
                }

                config.insert(token, next.unwrap());
            }
        }

        let event = config
            .iter()
            .find(|(key, _)| key.to_string() == "event")
            .map(|(_, value)| value);

        if event.is_none() {
            return Err(Error::new_spanned(
                signature.fn_token,
                "expected `event` attribute",
            ));
        }

        let authentication = config
            .clone()
            .into_iter()
            .find(|(key, _)| key.to_string() == "authentication")
            .map(|(_, value)| value)
            .unwrap_or(Lit::Bool(LitBool::new(true, Span::call_site())));

        let inputs = &signature.inputs;

        if inputs.len() > 2 {
            return Err(Error::new_spanned(inputs, "expected 2 arguments"));
        }

        let mut parameters = quote!();

        for i in inputs {
            match i {
                FnArg::Typed(pat) => {
                    let ty = &pat.ty;
                    let token = quote!(#ty).to_string();

                    if token.ends_with("Client") {
                        parameters = quote!(#parameters client.clone(),);
                    } else {
                        parameters =
                            quote!(#parameters ::serde_json::from_value(payload).unwrap(),);
                    }
                }
                _ => (),
            }
        }

        let name = &signature.ident;
        let event = event.unwrap();
        let block = &self.item.block;
        let token = quote!(
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy, Debug)]
            pub struct #name;

            impl ::zed_websocket::handler::Handler for #name {
                fn event(&self) -> ::zed_websocket::Regex {
                    ::zed_websocket::Regex::new(#event).unwrap()
                }

                fn authentication(&self) -> bool {
                    #authentication
                }

                fn call(
                    &self,
                    client: ::zed_websocket::client::Client,
                    payload: ::serde_json::Value,
                ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::zed_websocket::Future<Output = ()>>> {
                    async fn #name(#inputs) {
                        #block
                    }

                    let client = client.clone();

                    ::std::boxed::Box::pin(async move {
                        #name(#parameters).await;
                    })
                }
            }
        );

        Ok(token.into())
    }
}
