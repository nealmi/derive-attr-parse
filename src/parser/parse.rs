use std::collections::HashMap;

use proc_macro2::Span;
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::{token, Attribute, DeriveInput, Error, Token};

use crate::internals::symbol::*;
use crate::internals::Ctxt;

// #[sim(ode_solver = "eula")] V
// #[sim(ode_solver(algo = "eula", steps = "10"))] V

// #[sim(ode_solver{algo:"eula", steps:"10"})] X
// #[sim(ode_solver(ty=String, steps = 10))] X
// #[sim(ode_solver("eula", "10"))] X
// #[sim(ode_solver["eula", "modified_newton"))] X

pub fn root_from_ast<'a>(
    cx: &Ctxt,
    input: DeriveInput,
    root: Symbol,
) -> Result<Container<'a>, Error> {
    //当一个枚举或结构体被标记为NON_EXHAUSTIVE时，它表示该类型的未来版本可能添加新的变体（对于枚举）或字段（对于结构体），
    // 而不会被视为破坏性更改。这意味着在编写match语句时，你应该使用_通配符来处理未来可能添加的变化。
    let mut _non_exhaustive = false;

    let mut all = HashMap::new();
    for attr in &input.attrs {
        //bool |= exp 表示将布尔变量（或表达式）的值与表达式的值进行逻辑或运算，并将结果赋值给布尔变量。
        // 具体而言，如果 bool 变量的值为 true，那么无论表达式 exp 的值是什么，bool 的值都将保持为 true。
        // 只有在 bool 的值为 false 时，才会考虑表达式 exp 的值。如果 exp 的值为 true，那么 bool 的值将变为 true。

        //只解析 root 类型的 attr
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
    let res = data_from_ast(cx, input, root);
    if let Some(data) = res {
        eprintln!("{root} {all:#?}");
        let item = Container {
            ident: input.ident.clone(),
            attrs: all,
            data,
            generics: &input.generics,
            original: input,
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

#[derive(Debug, Clone)]
pub enum Val {
    Str(String),
    Map(HashMap<String, Val>),
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
        //bool |= exp 表示将布尔变量（或表达式）的值与表达式的值进行逻辑或运算，并将结果赋值给布尔变量。
        // 具体而言，如果 bool 变量的值为 true，那么无论表达式 exp 的值是什么，bool 的值都将保持为 true。
        // 只有在 bool 的值为 false 时，才会考虑表达式 exp 的值。如果 exp 的值为 true，那么 bool 的值将变为 true。

        //只解析 root 类型的 attr
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

#[derive(Debug, Clone)]
pub struct Container<'a> {
    /// The struct or enum name (without generics).
    pub ident: syn::Ident,
    /// Attributes on the structure, parsed for Serde.
    pub attrs: HashMap<String, Val>,
    /// The contents of the struct or enum.
    pub data: Data<'a>,
    /// Any generics on the struct or enum.
    pub generics: &'a syn::Generics,
    /// Original input.
    pub original: &'a syn::DeriveInput,
}

/// The fields of a struct or enum.
///
/// Analogous to `syn::Data`.
#[derive(Debug, Clone)]
pub enum Data<'a> {
    Enum(Vec<Variant<'a>>),
    Struct(Style, Vec<Field<'a>>),
}

#[derive(Debug, Clone, Copy)]
pub enum Style {
    /// Named fields.
    Struct,
    /// Many unnamed fields.
    Tuple,
    /// One unnamed field.
    Newtype,
    /// No fields.
    Unit,
}

/// A variant of an enum.
#[derive(Debug, Clone)]
pub struct Variant<'a> {
    pub ident: syn::Ident,
    pub attrs: HashMap<String, Val>,
    pub style: Style,
    pub fields: Vec<Field<'a>>,
    pub original: &'a syn::Variant,
}

/// A field of a struct.
#[derive(Debug, Clone)]
pub struct Field<'a> {
    pub member: syn::Member,
    pub attrs: HashMap<String, Val>,
    pub ty: &'a syn::Type,
    pub original: &'a syn::Field,
}
