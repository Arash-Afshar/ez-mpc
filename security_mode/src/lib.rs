#![feature(box_patterns)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn security_mode(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_ast = parse_macro_input!(attr as syn::Meta);
    let security_mode: String = if let syn::Meta::Path(syn::Path { segments, .. }) = attr_ast {
        assert!(segments.len() == 1);
        let syn::PathSegment { ident, .. } = &segments[0];
        ident.to_string()
    } else {
        unimplemented!();
    };

    let item_ast = parse_macro_input!(item as ItemFn);

    let find_target = |func| match func {
        syn::Expr::Path(syn::ExprPath {
            path: syn::Path { segments, .. },
            ..
        }) => segments.into_iter().fold(false, |acc, s| {
            let syn::PathSegment { ident, .. } = s;
            if ident.to_string() == "assign" {
                true
            } else {
                acc
            }
        }),
        _ => false,
    };

    let rewrite_expr =
        |attrs,
         func: syn::Expr,
         paren_token,
         args: syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>| {
            let mut args = args;
            let comma: syn::token::Comma = syn::token::Comma {
                spans: [Span::call_site()],
            };
            args.push_punct(comma);
            args.push_value(syn::Expr::Lit(syn::ExprLit {
                attrs: vec![],
                lit: syn::Lit::Str(syn::LitStr::new(&*security_mode, Span::call_site())),
            }));
            syn::Expr::Call(syn::ExprCall {
                attrs,
                func: Box::new(func),
                paren_token,
                args,
            })
        };

    let rewrite_semi = |attrs,
                        func: syn::Expr,
                        paren_token,
                        args: syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>,
                        semi_colon: syn::token::Semi| {
        syn::Stmt::Semi(rewrite_expr(attrs, func, paren_token, args), semi_colon)
    };

    let modified_statements = item_ast
        .block
        .stmts
        .into_iter()
        .map(|f| {
            let a = f.clone();
            match a {
                syn::Stmt::Local(syn::Local {
                    attrs,
                    let_token,
                    pat,
                    init: Some((eq_sign, box expr)),
                    semi_token,
                }) => {
                    let local_attrs = attrs;
                    let _expr = expr.clone();
                    match _expr {
                        syn::Expr::Call(syn::ExprCall {
                            box func,
                            attrs,
                            paren_token,
                            args,
                        }) => {
                            if find_target(func) {
                                let modified_expr =
                                    syn::Expr::Call(rewrite_expr(attrs, func, paren_token, args));
                                syn::Stmt::Local(syn::Local {
                                    attrs: local_attrs,
                                    let_token,
                                    pat,
                                    init: Some((eq_sign, Box::new(modified_expr))),
                                    semi_token,
                                })
                            } else {
                                f
                            }
                        }
                        _ => f,
                    }
                }
                syn::Stmt::Expr(syn::Expr::Call(syn::ExprCall {
                    box func,
                    attrs,
                    paren_token,
                    args,
                })) => {
                    if find_target(func.clone()) {
                        syn::Stmt::Expr(rewrite_expr(attrs, func, paren_token, args))
                    } else {
                        f
                    }
                }
                syn::Stmt::Semi(
                    syn::Expr::Call(syn::ExprCall {
                        box func,
                        attrs,
                        paren_token,
                        args,
                    }),
                    semi_colon,
                ) => {
                    if find_target(func.clone()) {
                        rewrite_semi(attrs, func, paren_token, args, semi_colon)
                    } else {
                        f
                    }
                }
                _ => f,
            }
        })
        .collect::<Vec<syn::Stmt>>();

    let modified_block = syn::Block {
        brace_token: item_ast.block.brace_token,
        stmts: modified_statements,
    };

    let modified_func: syn::ItemFn = syn::ItemFn {
        attrs: item_ast.attrs,
        vis: item_ast.vis,
        sig: item_ast.sig,
        block: std::boxed::Box::<syn::Block>::new(modified_block),
    };

    modified_func.to_token_stream().into()
}
