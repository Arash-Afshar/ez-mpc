#![feature(box_patterns)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn security_mode(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_ast = parse_macro_input!(attr as syn::Meta);
    let securit_mode: String = if let syn::Meta::Path(syn::Path { segments, .. }) = attr_ast {
        assert!(segments.len() == 1);
        let syn::PathSegment { ident, .. } = &segments[0];
        ident.to_string()
    } else {
        unimplemented!();
    };

    let item_ast = parse_macro_input!(item as ItemFn);

    let target_seen = |segments: &syn::punctuated::Punctuated<syn::PathSegment, _>| {
        segments.into_iter().fold(false, |acc, s| {
            let syn::PathSegment { ident, .. } = s;
            if ident.to_string() == "assign" {
                true
            } else {
                acc
            }
        })
    };

    let modified_statements = item_ast
        .block
        .stmts
        .into_iter()
        .map(|f| {
            if let syn::Stmt::Semi(syn::Expr::Call(syn::ExprCall { func, .. }), ..) = &f {
                match func {
                    box e => {
                        if let syn::Expr::Path(syn::ExprPath {
                            path: syn::Path { segments, .. },
                            ..
                        }) = e
                        {
                            if target_seen(segments) {
                                if let syn::Stmt::Semi(
                                    syn::Expr::Call(syn::ExprCall {
                                        attrs,
                                        func,
                                        paren_token,
                                        args,
                                    }),
                                    s,
                                ) = f.clone()
                                {
                                    let mut args = args;
                                    let comma: syn::token::Comma = syn::token::Comma {
                                        spans: [Span::call_site()],
                                    };
                                    args.push_punct(comma);
                                    args.push_value(syn::Expr::Lit(syn::ExprLit {
                                        attrs: vec![],
                                        lit: syn::Lit::Str(syn::LitStr::new(
                                            &*securit_mode,
                                            Span::call_site(),
                                        )),
                                    }));
                                    syn::Stmt::Semi(
                                        syn::Expr::Call(syn::ExprCall {
                                            attrs,
                                            func,
                                            paren_token,
                                            args,
                                        }),
                                        s,
                                    )
                                } else {
                                    f
                                }
                            } else {
                                f
                            }
                        } else {
                            f
                        }
                    }
                }
            } else {
                f
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
