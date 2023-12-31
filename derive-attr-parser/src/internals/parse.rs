use std::collections::HashMap;

use proc_macro2::Span;
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::{token, Attribute, DeriveInput, Error, Token};

use crate::internals::ast::{Container, Data, Field, Style, Symbol, Val, Variant};
use crate::internals::ctxt::Ctxt;

// The attr should keep simple as following supported literal
// you can process string val as you want after extract the meta from attr,
// NOTE: everything in val should treated as string.
// supported:
// #[sim(ode_solver = "eula")] V
// #[sim(ode_solver(algo = "eula", steps = "10"))] V
// #[sim(ode_solver(algorithms = r#"["eula", "newton", "test"]"#))] V
// #[sim(ode_solver(algo = r#"{algo:"eula", steps:10}"#))] V
// unsupported:
// #[sim(ode_solver{algo:"eula", steps:"10"})] X
// #[sim(ode_solver={algo:"eula", steps:"10"})] X
// #[sim(ode_solver(ty=String, steps = 10))] X
// #[sim(ode_solver("eula", "10"))] X
// #[sim(ode_solver["eula", "modified_newton"))] X
// #[sim(ode_solver=["eula", "modified_newton"))] X
pub fn from_ast<'a>(
    cx: &Ctxt,
    input: &'a DeriveInput,
    root: Symbol,
) -> Result<Container<'a>, Error> {
    container_from_ast(cx, input, root)
}

fn container_from_ast<'a>(
    cx: &Ctxt,
    input: &'a DeriveInput,
    root: Symbol,
) -> Result<Container<'a>, Error> {
    let attrs = parse_attrs(cx, &input.attrs, root)?;
    let res = data_from_ast(cx, &input, root);
    if let Some(data) = res {
        //eprintln!("{root} {attrs:#?}");
        let item = Container {
            ident: input.ident.clone(),
            attrs,
            data,
            generics: &input.generics,
            original: &input,
        };
        Ok(item)
    } else {
        Err(Error::new(
            Span::call_site(),
            "Data is none#container_from_ast",
        ))
    }
}

fn parse_sub_attrs(cx: &Ctxt, meta: &ParseNestedMeta) -> syn::Result<HashMap<String, Val>> {
    let lookahead = meta.input.lookahead1();
    let mut attrs = HashMap::new();
    if let Some(ident) = meta.path.get_ident() {
        let key = ident.to_string();
        // #[sim(ode_solver = "eula")]
        if lookahead.peek(Token![=]) {
            attrs.insert(key, get_val_str(&meta)?);
        } else if lookahead.peek(token::Paren) {
            // #[sim(ode_solver(algo = "eula", steps = "10"))]
            let mut all_sub_attrs = HashMap::new();
            if let Err(err) = meta.parse_nested_meta(|m| {
                merge_map(cx, parse_sub_attrs(cx, &m)?, &mut all_sub_attrs);
                Ok(())
            }) {
                cx.syn_error(err);
            }
            attrs.insert(key, Val::Map(all_sub_attrs));
        } else if lookahead.peek(Token![:]) {
            attrs.insert(key, get_val_str(&meta)?);
        } else {
            attrs.insert(key, Val::Empty);
        }
    } else {
        let msg = format!("no ident found #parse_sub_attrs");
        let err = Error::new(Span::call_site(), msg);
        cx.syn_error(err);
    }

    Ok(attrs)
}

fn get_val_str(meta: &ParseNestedMeta) -> syn::Result<Val> {
    if let Err(eq) = meta.input.parse::<Token![=]>() {
        if let Err(_ec) = meta.input.parse::<Token![:]>() {
            let ident = meta.path.get_ident();
            let msg = format!("expect either '=' or ':' after ident {ident:?} #get_val_str");
            let err = Error::new(eq.span(), msg);
            return Err(err);
        }
    }
    let expr: syn::Expr = meta.input.parse()?;
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
        Ok(Val::Str(lit.clone().value()))
    } else {
        Ok(Val::Str("".to_string()))
    }
}

fn data_from_ast<'a>(cx: &Ctxt, input: &'a DeriveInput, root: Symbol) -> Option<Data<'a>> {
    let data = match &input.data {
        syn::Data::Enum(data) => Data::Enum(enum_from_ast(cx, &data.variants, root)),
        syn::Data::Struct(data) => {
            let (style, fields) = struct_from_ast(cx, &data.fields, root);
            Data::Struct(style, fields)
        }
        syn::Data::Union(_) => {
            let msg = format!("Does not support derive for unions#data_from_ast");
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

fn filed_from_ast(
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

fn variant_from_ast(cx: &Ctxt, variant: &syn::Variant, root: Symbol) -> HashMap<String, Val> {
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

fn merge_map(_cx: &Ctxt, from: HashMap<String, Val>, to: &mut HashMap<String, Val>) {
    for (k, v) in from {
        if let Some(x) = to.get(&k) {
            eprintln!("duplicated key {{{k}}} #merge_map");
            let mut vs = vec![];
            match x {
                Val::Vec(v) => {
                    vs.append(&mut v.to_vec())
                }
                _ => {
                    vs.push(x.clone());
                    vs.push(v);
                }
            };
            to.insert(k, Val::Vec(vs));
            // let msg = format!("duplicated key {{{k}}}#merge_map");
            // cx.syn_error(Error::new(Span::call_site(), msg));
        } else {
            to.insert(k, v);
        }
    }
}
