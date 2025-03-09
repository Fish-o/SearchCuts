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
    Word,
    Text,
    Number,
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
                let ident = rule.parse::<Ident>()?;
                match ident.to_string().as_ref() {
                    "word" => ArgType::Word,
                    "text" => ArgType::Text,
                    "number" => ArgType::Number,
                    m => panic!("Unsupported argtype '{m}'"),
                }
                // ArgType::Type(rule.parse::<TypePath>()?)
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
        let mut arg_option = None;
        for (rule, res) in rules {
            let res = match res {
                MacroRuleResult::Fn(block) => quote! {
                   #block
                }
                .into(),
                MacroRuleResult::Rule(ident) => {
                    let ident = format_ident!("layer_{ident}");
                    quote! {
                        #ident(path, input, activate)
                    }
                }
            };
            let c = match rule {
                ArgType::Const(c) => c,
                a => {
                    if arg_option.is_some() {
                        panic!("Argument overloading is not supported.")
                    }
                    arg_option = Some((a, res));
                    continue;
                }
            };
            options.push(
                quote! {
                    (#c.to_owned(), Box::new(|path, input| #res))
                }
                .into(),
            )
        }
        let layer = format_ident!("layer_{layer}");
        if arg_option.is_some() && !options.is_empty() {
            panic!("Can not have both an argument and a constant as options.")
        }
        match arg_option {
            Some((ArgType::Text, res))=>{
                result.push(quote! {
                    fn #layer(path: String, input: &str, activate: bool) -> Vec<MetaResult> {
                        #res
                    }
                });
            },
            Some((_, res)) =>{
                todo!();
                result.push(quote! {
                    fn #layer(path: String, input: &str) -> Vec<MetaResult> {
                        vec![MetaResult {
                            name: format!("{} {n}", path.trim()),
                            description: format!("{} {input}", path.trim()),
                            path: format!("{path} {input}"),
                            icon: None
                        }]
                    }
                });
            }
            None => result.push(quote! {
                fn #layer(path: String, input: &str, activate: bool) -> Vec<MetaResult> {
                    let options: Vec<(String, Box<dyn Fn(String, &str) -> Vec<MetaResult>>)> = vec![#(#options),*];
                    let data = options
                        .iter()
                        .filter(|(n, c)| input.starts_with(n))
                        .collect::<Vec<_>>();
                    if activate && data.len() != 1 {
                        return vec![]
                    }
                    println!("-'{path}' -'{input}' {} {}", options.len(), data.len());
                    if data.len() == 1 {
                        let d = &data[0];
                        let new_input = (&input[d.0.len()..]).trim();
                        let rem_len = input.len() - new_input.len();
                        let path = format!("{path}{}", &input[..rem_len]);
                        d.1(path, new_input)
                    } else if data.len() == 0{
                        let r = options
                            .iter()
                            .filter(|(n, c)| n.starts_with(input))
                            .map(|(n,_)| MetaResult {
                                name: format!("{} {n}", path.trim()),
                                description: format!("{} {input}", path.trim()),
                                path: format!("{path} {input}"),
                                icon: None
                            })
                            .collect::<Vec<_>>();
                        dbg!(&r);
                        r
                    } else {

                        vec![]
                    }
                }
            }),
        }
    }

    quote! {
        enum ParsedArg {
            Text(String),
            Word(String),
            Number(String),
        }
        #(#result)*
    }
    .into()
}
