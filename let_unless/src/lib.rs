use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream};

#[proc_macro]
pub fn let_unless(stream: TokenStream) -> TokenStream {
    let LetUnless {
        ident,
        pat,
        body
    } = syn::parse_macro_input!(stream as LetUnless);

    quote::quote!{
        if !matches!(#ident, #pat) {
            #ident
        } else #body
    }.into()
}

struct LetUnless {
    ident: syn::Ident,
    pat: syn::Pat,
    body: syn::Block
}

impl Parse for LetUnless {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        input.parse::<kw::unless>()?;
        let pat = syn::Pat::parse_multi(input)?;
        input.parse::<syn::Token![=>]>()?;
        
        let body = input.parse()?;
        return Ok(LetUnless {
            ident,
            pat,
            body
        });
    }
}

mod kw {
    syn::custom_keyword!(unless);
}