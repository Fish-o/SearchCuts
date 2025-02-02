use core::panic;
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{
    bracketed,
    parse::Parse,
    parse_macro_input,
    token::{Brace, Group},
    Block, Ident, ItemFn, LitStr, Token, TypePath,
};

mod utils {
    use proc_macro::token_stream::IntoIter;
}

#[derive(Debug, Clone)]
enum ArgType {
    Const(String),
    Type(TypePath),
}

#[derive(Debug)]
struct MacroRules {
    rules: HashMap<Ident, Vec<(ArgType, MacroRuleResult)>>,
}

type FunctionType = Block;
#[derive(Debug)]
enum MacroRuleResult {
    Fn(FunctionType),
    Rule(Ident),
}

impl Parse for MacroRules {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut rules = HashMap::new();
        while input.peek(Ident) {
            let layer = input.parse::<syn::Ident>()?;
            let rule;
            bracketed!(rule in input);
            let rule = if rule.peek(LitStr) {
                ArgType::Const(rule.parse::<LitStr>()?.value())
            } else if rule.peek(Ident) {
                ArgType::Type(rule.parse::<TypePath>()?)
            } else {
                // TODO: Add regex
                panic!("Invalid parser rule.")
            };

            input.parse::<Token![=]>()?;
            let result = if input.peek(Brace) {
                MacroRuleResult::Fn(input.parse::<FunctionType>()?)
            } else {
                MacroRuleResult::Rule(input.parse::<syn::Ident>()?)
            };
            input.parse::<Token![;]>()?;
            rules.entry(layer).or_insert(vec![]).push((rule, result));
        }
        Ok(MacroRules { rules })
    }
}
#[proc_macro]
pub fn create_grammar(tokens: TokenStream) -> TokenStream {
    dbg!(&tokens);
    let parsed = parse_macro_input!(tokens as MacroRules);
    dbg!(&parsed);
    let mut result = vec![quote! {}];

    for (layer, rules) in parsed.rules {
        let mut options: Vec<proc_macro2::TokenStream> = vec![];
        for (rule, res) in rules {
            let c = match rule {
                ArgType::Const(c) => c,
                _ => todo!(),
            };
            let res = match res {
                MacroRuleResult::Fn(block) => quote! {
                   #block
                }
                .into(),
                MacroRuleResult::Rule(ident) => {
                    let ident = format_ident!("layer_{ident}");
                    quote! {

                        #ident(path)
                    }
                }
            };

            options.push(
                quote! {
                    (#c.to_owned(), |path| #res)
                }
                .into(),
            )
        }
        let layer = format_ident!("layer_{layer}");
        result.push(quote! {
            fn #layer(input: &str) -> Option<MetaResult> {
                let options = [#(#options),*];
                let data = options
                    .into_iter()
                    .filter(|(n, c)| input.starts_with(n))
                    .collect::<Vec<_>>();
                if data.len() == 1 {
                    let d = &data[0];
                    let new_input = &input[d.0.len()..];
                    d.1(new_input.trim())
                } else {
                    None
                }
            }
        });
    }

    quote! {
        #(#result)*
    }
    .into()
}
