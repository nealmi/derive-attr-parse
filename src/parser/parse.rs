use std::collections::HashMap;

use proc_macro2::Span;
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::{token, Attribute, DeriveInput, Error, Token};
use crate::parser::ast::{Container, Data, Field, Style, Val, Variant};

use crate::parser::symbol::*;
use crate::parser::ctxt::Ctxt;

// #[sim(ode_solver = "eula")] V
// #[sim(ode_solver(algo = "eula", steps = "10"))] V

// #[sim(ode_solver{algo:"eula", steps:"10"})] X
// #[sim(ode_solver(ty=String, steps = 10))] X
// #[sim(ode_solver("eula", "10"))] X
// #[sim(ode_solver["eula", "modified_newton"))] X

pub fn root_from_ast<'a>(
    cx: &Ctxt,
    input: &'a DeriveInput,
    root: Symbol,
) -> Result<Container<'a>, Error> {
    let mut _non_exhaustive = false;

    let mut all = HashMap::new();
    for attr in &input.attrs {
        if attr.path() != root {
            _non_exhaustive |=
                matches!(&attr.meta, syn::Meta::Path(path) if path == NON_EXHAUSTIVE);
            continue;
        }
        if let syn::Meta::List(meta) = &attr.meta {
            if meta.tokens.is_empty() {
                continue;
            }
        }

        let mut attrs = HashMap::new();
        if let Err(err) = attr.parse_nested_meta(|meta| {
            // 解析子 attr
            let sub_attrs = parse_sub_attrs(cx, &meta);
            merge_map(cx, sub_attrs?, &mut attrs);
            Ok(())
        }) {
            cx.syn_error(err);
        }
        merge_map(cx, attrs, &mut all)
    }
    let res = data_from_ast(cx, &input, root);
    if let Some(data) = res {
        eprintln!("{root} {all:#?}");
        let item = Container {
            ident: input.ident.clone(),
            attrs: all,
            data,
            generics: &input.generics,
            original: &input,
        };

        Ok(item)
    } else {
        Err(Error::new(Span::call_site(), "Data is none"))
    }
}

pub fn merge_map(cx: &Ctxt, from: HashMap<String, Val>, to: &mut HashMap<String, Val>) {
    for (k, v) in from {
        if to.get(&k).is_some() {
            let msg = format!("duplicated key {{{k}}}");
            cx.syn_error(Error::new(Span::call_site(), msg));
        } else {
            to.insert(k, v);
        }
    }
}

pub fn parse_sub_attrs(cx: &Ctxt, meta: &ParseNestedMeta) -> syn::Result<HashMap<String, Val>> {
    let lookahead = meta.input.lookahead1();
    let mut idents = HashMap::new();
    let key = meta.path.get_ident().unwrap().to_string();

    // #[sim(ode_solver = "eula")]
    if lookahead.peek(Token![=]) {
        idents.insert(key, Val::Str(get_lit_str(&meta)?.unwrap()));
    } else if lookahead.peek(token::Paren) {
        // #[sim(ode_solver(algo = "eula", steps = "10"))]
        let mut subs = HashMap::new();

        if let Err(err) = meta.parse_nested_meta(|m| {
            let sub_key = m.path.get_ident().unwrap().to_string();

            subs.insert(sub_key, Val::Str(get_lit_str(&m)?.unwrap()));
            Ok(())
        }) {
            cx.syn_error(err);
        }
        idents.insert(key, Val::Map(subs));
    }

    Ok(idents)
}



fn get_lit_str(meta: &ParseNestedMeta) -> syn::Result<Option<String>> {
    let expr: syn::Expr = meta.value()?.parse()?;
    let mut value = &expr;
    while let syn::Expr::Group(e) = value {
        value = &e.expr;
    }
    if let syn::Expr::Lit(syn::ExprLit {
                              lit: syn::Lit::Str(lit),
                              ..
                          }) = value
    {
        let suffix = lit.suffix();
        if !suffix.is_empty() {}
        Ok(Some(lit.clone().value()))
    } else {
        Ok(Some("".to_string()))
    }
}

fn data_from_ast<'a>(
    cx: &Ctxt,
    input: &'a DeriveInput,
    root: Symbol,
) -> Option<Data<'a>> {
    let data = match &input.data {
        syn::Data::Enum(data) => Data::Enum(enum_from_ast(cx, &data.variants, root)),
        syn::Data::Struct(data) => {
            let (style, fields) = struct_from_ast(cx, &data.fields, root);
            Data::Struct(style, fields)
        }
        syn::Data::Union(_) => {
            let msg = format!("Does not support derive for unions");
            cx.error_spanned_by(input, &msg);
            return None;
        }
    };

    Some(data)
}

fn struct_from_ast<'a>(
    cx: &Ctxt,
    fields: &'a syn::Fields,
    root: Symbol,
) -> (Style, Vec<Field<'a>>) {
    match fields {
        syn::Fields::Named(fields) => (Style::Struct, fields_from_ast(cx, &fields.named, root)),
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            (Style::Newtype, fields_from_ast(cx, &fields.unnamed, root))
        }
        syn::Fields::Unnamed(fields) => (Style::Tuple, fields_from_ast(cx, &fields.unnamed, root)),
        syn::Fields::Unit => (Style::Unit, Vec::new()),
    }
}

fn fields_from_ast<'a>(
    cx: &Ctxt,
    fields: &'a Punctuated<syn::Field, Token![,]>,
    root: Symbol,
) -> Vec<Field<'a>> {
    fields
        .iter()
        .enumerate()
        .map(|(i, field)| Field {
            member: match &field.ident {
                Some(ident) => syn::Member::Named(ident.clone()),
                None => syn::Member::Unnamed(i.into()),
            },
            attrs: filed_from_ast(cx, i, field, root),
            ty: &field.ty,
            original: field,
        })
        .collect()
}

fn parse_attrs(
    cx: &Ctxt,
    attrs: &Vec<Attribute>,
    root: Symbol,
) -> syn::Result<HashMap<String, Val>> {
    let mut all = HashMap::new();
    for attr in attrs {
        if attr.path() != root {
            continue;
        }
        if let syn::Meta::List(meta) = &attr.meta {
            if meta.tokens.is_empty() {
                continue;
            }
        }
        let mut attrs = HashMap::new();
        if let Err(err) = attr.parse_nested_meta(|meta| {
            // 解析子 attr
            let sub_attrs = parse_sub_attrs(cx, &meta);
            merge_map(cx, sub_attrs?, &mut attrs);
            Ok(())
        }) {
            cx.syn_error(err);
        }
        merge_map(cx, attrs, &mut all)
    }

    // eprintln!("{root} {all:#?}");
    Ok(all)
}

pub fn filed_from_ast(
    cx: &Ctxt,
    _index: usize,
    field: &syn::Field,
    root: Symbol,
) -> HashMap<String, Val> {
    match parse_attrs(cx, &field.attrs, root) {
        Ok(m) => m,
        Err(e) => {
            cx.error_spanned_by(field, e);
            HashMap::new()
        }
    }
}

pub fn variant_from_ast(cx: &Ctxt, variant: &syn::Variant, root: Symbol) -> HashMap<String, Val> {
    match parse_attrs(cx, &variant.attrs, root) {
        Ok(map) => map,
        Err(e) => {
            cx.syn_error(e);
            HashMap::new()
        }
    }
}

fn enum_from_ast<'a>(
    cx: &Ctxt,
    variants: &'a Punctuated<syn::Variant, Token![,]>,
    root: Symbol,
) -> Vec<Variant<'a>> {
    let variants: Vec<Variant> = variants
        .iter()
        .map(|variant| {
            let attrs = variant_from_ast(cx, variant, root);
            let (style, fields) = struct_from_ast(cx, &variant.fields, root);
            Variant {
                ident: variant.ident.clone(),
                attrs,
                style,
                fields,
                original: variant,
            }
        })
        .collect();
    variants
}
