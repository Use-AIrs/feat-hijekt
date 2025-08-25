use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    visit_mut::VisitMut,
    Block, Error, Fields, Ident, Item, ItemFn, ItemStruct, Lit, LitStr, Pat, Result, Stmt, Token,
};

#[derive(Debug, Clone, Default)]
struct HijektConfig {
    feat: String,
    begin: Vec<String>,
    begin_with: Vec<String>,
    end: Vec<String>,
    rm: Vec<String>,
    replace: Option<String>,
    add: Vec<String>,
}

impl HijektConfig {
    fn feature_flag(&self) -> String {
        self.feat.clone()
    }

    fn is_simple_feature_only(&self) -> bool {
        self.begin.is_empty()
            && self.begin_with.is_empty()
            && self.end.is_empty()
            && self.rm.is_empty()
            && self.replace.is_none()
            && self.add.is_empty()
    }

    fn parse_meta_item(&mut self, meta: syn::meta::ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("feat") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            self.feat = lit.value();
            return Ok(());
        }

        if meta.path.is_ident("begin") {
            if meta.input.peek(Token![=]) {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                self.begin.push(lit.value());
            } else if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    let lit: LitStr = nested.input.parse()?;
                    self.begin.push(lit.value());
                    Ok(())
                })?;
            }
            return Ok(());
        }

        if meta.path.is_ident("begin_with") {
            if meta.input.peek(Token![=]) {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                self.begin_with.push(lit.value());
            } else if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    let lit: LitStr = nested.input.parse()?;
                    self.begin_with.push(lit.value());
                    Ok(())
                })?;
            }
            return Ok(());
        }

        if meta.path.is_ident("end") {
            if meta.input.peek(Token![=]) {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                self.end.push(lit.value());
            } else if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    let lit: LitStr = nested.input.parse()?;
                    self.end.push(lit.value());
                    Ok(())
                })?;
            }
            return Ok(());
        }

        if meta.path.is_ident("rm") {
            if meta.input.peek(Token![=]) {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                self.rm.push(lit.value());
            } else if meta.input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in meta.input);
                let items: Punctuated<Lit, Token![,]> =
                    content.parse_terminated(Lit::parse, Token![,])?;
                for item in items {
                    if let Lit::Str(litstr) = item {
                        self.rm.push(litstr.value());
                    }
                }
            }
            return Ok(());
        }

        if meta.path.is_ident("swap") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            self.replace = Some(lit.value());
            return Ok(());
        }

        if meta.path.is_ident("add") {
            if meta.input.peek(Token![=]) {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                self.add.push(lit.value());
            } else if meta.input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in meta.input);
                let items: Punctuated<Lit, Token![,]> =
                    content.parse_terminated(Lit::parse, Token![,])?;
                for item in items {
                    if let Lit::Str(litstr) = item {
                        self.add.push(litstr.value());
                    }
                }
            }
            return Ok(());
        }

        Err(meta.error("unrecognized hijekt attribute"))
    }
}

struct HijektArgs {
    config: HijektConfig,
}

impl Parse for HijektArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut config = HijektConfig::default();

        let metas = Punctuated::<syn::Meta, Token![,]>::parse_terminated(input)?;

        for meta in metas {
            match meta {
                syn::Meta::NameValue(nv) => {
                    if nv.path.is_ident("feat") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let syn::Lit::Str(s) = &lit.lit {
                                config.feat = s.value();
                            }
                        }
                    } else if nv.path.is_ident("begin") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let syn::Lit::Str(s) = &lit.lit {
                                config.begin.push(s.value());
                            }
                        }
                    } else if nv.path.is_ident("begin_with") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let syn::Lit::Str(s) = &lit.lit {
                                config.begin_with.push(s.value());
                            }
                        }
                    } else if nv.path.is_ident("end") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let syn::Lit::Str(s) = &lit.lit {
                                config.end.push(s.value());
                            }
                        }
                    } else if nv.path.is_ident("swap") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let syn::Lit::Str(s) = &lit.lit {
                                config.replace = Some(s.value());
                            }
                        }
                    } else if nv.path.is_ident("rm") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let syn::Lit::Str(s) = &lit.lit {
                                config.rm.push(s.value());
                            }
                        }
                    } else if nv.path.is_ident("add") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let syn::Lit::Str(s) = &lit.lit {
                                config.add.push(s.value());
                            }
                        }
                    }
                }
                syn::Meta::List(list) => {
                    if list.path.is_ident("rm") {
                        let nested = list
                            .parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)?;
                        for lit in nested {
                            config.rm.push(lit.value());
                        }
                    } else if list.path.is_ident("begin") {
                        let nested = list
                            .parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)?;
                        for lit in nested {
                            config.begin.push(lit.value());
                        }
                    } else if list.path.is_ident("begin_with") {
                        let nested = list
                            .parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)?;
                        for lit in nested {
                            config.begin_with.push(lit.value());
                        }
                    } else if list.path.is_ident("end") {
                        let nested = list
                            .parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)?;
                        for lit in nested {
                            config.end.push(lit.value());
                        }
                    } else if list.path.is_ident("add") {
                        let nested = list
                            .parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)?;
                        for lit in nested {
                            config.add.push(lit.value());
                        }
                    }
                }
                syn::Meta::Path(path) => {
                    return Err(Error::new_spanned(path, "expected key-value or list"));
                }
            }
        }

        if config.feat.is_empty() {
            return Err(Error::new(Span::call_site(), "feat attribute is required"));
        }

        Ok(HijektArgs { config })
    }
}

#[proc_macro_attribute]
pub fn hijekt(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as HijektArgs);
    let config = args.config;

    if config.is_simple_feature_only() {
        let feat_flag = config.feature_flag();
        let item = parse_macro_input!(input as Item);
        return TokenStream::from(quote! {
            #[cfg(feature = #feat_flag)]
            #item
        });
    }

    if let Ok(item_fn) = syn::parse::<ItemFn>(input.clone()) {
        return handle_function(config, item_fn);
    }

    if let Ok(item_struct) = syn::parse::<ItemStruct>(input.clone()) {
        return handle_struct(config, item_struct);
    }

    let feat_flag = config.feature_flag();
    let item = parse_macro_input!(input as Item);
    TokenStream::from(quote! {
        #[cfg(feature = #feat_flag)]
        #item
    })
}

fn handle_function(config: HijektConfig, func: ItemFn) -> TokenStream {
    let feat_flag = config.feature_flag();
    let original = func.clone();

    if let Some(replace_fn) = &config.replace {
        let replace_ident: Ident = syn::parse_str(replace_fn).unwrap();
        let vis = &func.vis;
        let sig = &func.sig;
        let attrs = &func.attrs;

        let args: Vec<_> = sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                    syn::Pat::Ident(ident) => Some(quote! { #ident }),
                    _ => None,
                },
                syn::FnArg::Receiver(_) => Some(quote! { self }),
            })
            .collect();

        let begin_calls: Vec<Stmt> = config
            .begin
            .iter()
            .map(|begin_fn| {
                let begin_ident: Ident = syn::parse_str(begin_fn).unwrap();
                parse_quote! { #begin_ident(); }
            })
            .collect();

        let begin_with_calls: Vec<Stmt> = config
            .begin_with
            .iter()
            .map(|begin_fn| {
                let begin_ident: Ident = syn::parse_str(begin_fn).unwrap();
                let ref_args: Vec<_> = sig
                    .inputs
                    .iter()
                    .filter_map(|arg| match arg {
                        syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                            syn::Pat::Ident(ident) => Some(quote! { &#ident }),
                            _ => None,
                        },
                        syn::FnArg::Receiver(_) => Some(quote! { &self }),
                    })
                    .collect();
                parse_quote! { #begin_ident(#(#ref_args),*); }
            })
            .collect();

        let end_calls: Vec<Stmt> = config
            .end
            .iter()
            .map(|end_fn| {
                let end_ident: Ident = syn::parse_str(end_fn).unwrap();
                parse_quote! { #end_ident(); }
            })
            .collect();

        let has_return = !matches!(sig.output, syn::ReturnType::Default);
        let swap_body = if has_return {
            if !end_calls.is_empty() {
                quote! {
                    #(#begin_calls)*
                    #(#begin_with_calls)*
                    let __result = #replace_ident(#(#args),*);
                    #(#end_calls)*
                    __result
                }
            } else {
                quote! {
                    #(#begin_calls)*
                    #(#begin_with_calls)*
                    #replace_ident(#(#args),*)
                }
            }
        } else {
            quote! {
                #(#begin_calls)*
                #(#begin_with_calls)*
                #replace_ident(#(#args),*);
                #(#end_calls)*
            }
        };

        return TokenStream::from(quote! {
            #(#attrs)*
            #[cfg(feature = #feat_flag)]
            #vis #sig {
                #swap_body
            }

            #[cfg(not(feature = #feat_flag))]
            #original
        });
    }

    let mut modified = func.clone();

    for rm_target in &config.rm {
        let mut remover = ItemRemover {
            targets: vec![rm_target.clone()],
        };
        remover.visit_block_mut(&mut modified.block);
    }

    for begin_fn in config.begin.iter().rev() {
        let begin_ident: Ident = syn::parse_str(begin_fn).unwrap();
        modified
            .block
            .stmts
            .insert(0, parse_quote! { #begin_ident(); });
    }

    for begin_fn in config.begin_with.iter().rev() {
        let begin_ident: Ident = syn::parse_str(begin_fn).unwrap();
        let ref_args: Vec<_> = func
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                    syn::Pat::Ident(ident) => Some(quote! { &#ident }),
                    _ => None,
                },
                syn::FnArg::Receiver(_) => Some(quote! { &self }),
            })
            .collect();
        modified
            .block
            .stmts
            .insert(0, parse_quote! { #begin_ident(#(#ref_args),*); });
    }

    if !config.end.is_empty() {
        inject_at_end(&mut modified.block, &config.end);
    }

    TokenStream::from(quote! {
        #[cfg(feature = #feat_flag)]
        #modified

        #[cfg(not(feature = #feat_flag))]
        #original
    })
}

fn handle_struct(config: HijektConfig, item: ItemStruct) -> TokenStream {
    let feat_flag = config.feature_flag();
    let original = item.clone();
    let mut modified = item.clone();

    for rm_field in &config.rm {
        if let Fields::Named(ref mut fields) = modified.fields {
            fields.named = fields
                .named
                .iter()
                .filter(|f| {
                    f.ident
                        .as_ref()
                        .map(|i| i.to_string() != *rm_field)
                        .unwrap_or(true)
                })
                .cloned()
                .collect();
        }
    }

    for add_spec in &config.add {
        if let Fields::Named(ref mut fields) = modified.fields {
            if add_spec.contains(':') {
                // Parse "field_name: Type"
                let parts: Vec<&str> = add_spec.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let field_name = parts[0].trim();
                    let type_str = parts[1].trim();

                    if let Ok(field_ident) = syn::parse_str::<Ident>(field_name) {
                        if let Ok(field_type) = syn::parse_str::<syn::Type>(type_str) {
                            fields.named.push(parse_quote! {
                                pub #field_ident: #field_type
                            });
                        }
                    }
                }
            } else {
                // Only type specified, generate field name
                let sanitized_name = add_spec
                    .to_lowercase()
                    .replace("::", "_")
                    .replace('<', "_")
                    .replace('>', "")
                    .replace(' ', "")
                    .replace(',', "_");

                let field_name = format_ident!("hijekt_{}", sanitized_name);

                if let Ok(field_type) = syn::parse_str::<syn::Type>(add_spec) {
                    fields.named.push(parse_quote! {
                        pub #field_name: #field_type
                    });
                }
            }
        }
    }

    TokenStream::from(quote! {
        #[cfg(feature = #feat_flag)]
        #modified

        #[cfg(not(feature = #feat_flag))]
        #original
    })
}

fn inject_at_end(block: &mut Block, end_fns: &[String]) {
    let has_implicit_return = block
        .stmts
        .last()
        .map_or(false, |stmt| matches!(stmt, Stmt::Expr(_, None)));

    let end_calls: Vec<Stmt> = end_fns
        .iter()
        .map(|end_fn| {
            let end_ident: Ident = syn::parse_str(end_fn).unwrap();
            parse_quote! { #end_ident(); }
        })
        .collect();

    if has_implicit_return {
        if let Some(Stmt::Expr(expr, None)) = block.stmts.pop() {
            block.stmts.push(parse_quote! {
                let __hijekt_result = #expr;
            });

            block.stmts.extend(end_calls);

            block
                .stmts
                .push(Stmt::Expr(parse_quote! { __hijekt_result }, None));
        }
    } else {
        block.stmts.extend(end_calls);
    }
}

struct ItemRemover {
    targets: Vec<String>,
}

impl VisitMut for ItemRemover {
    fn visit_block_mut(&mut self, block: &mut Block) {
        block.stmts.retain(|stmt| match stmt {
            Stmt::Item(Item::Fn(func)) => !self.targets.contains(&func.sig.ident.to_string()),
            Stmt::Local(local) => {
                if let Pat::Ident(ident) = &local.pat {
                    !self.targets.contains(&ident.ident.to_string())
                } else {
                    true
                }
            }
            _ => true,
        });

        syn::visit_mut::visit_block_mut(self, block);
    }
}
