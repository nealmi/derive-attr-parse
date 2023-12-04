use std::collections::HashMap;
use std::fmt::{self, Display};
use proc_macro2::Span;
use syn::{Expr, Ident, Path};

/// The root Container for all
///
/// let root:Container  = from_ast(...)
///
#[derive(Debug, Clone)]
pub struct Container<'a> {
    /// The struct or enum name (without generics).
    pub ident: syn::Ident,
    /// Attributes on the structure.
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

/// The root of helper attr, eg #[root(...)]
/// ```rust
/// use derive_attr_parser::Symbol;
/// const ROOT:Symbol = Symbol("root");
/// //let root:Container  = from_ast(.. ROOT);
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Symbol(pub &'static str);

impl From<String> for Symbol {
    fn from(value: String) -> Self {
        Symbol(value.leak())
    }
}
// impl<'a> From<&'a str> for Symbol {
//     fn from(value: &'a str) -> Self {
//         Symbol(value.to_string().leak())
//     }
// }

impl PartialEq<Symbol> for Ident {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl<'a> PartialEq<Symbol> for &'a Ident {
    fn eq(&self, word: &Symbol) -> bool {
        *self == word.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl<'a> PartialEq<Symbol> for &'a Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl<'a> PartialEq<Symbol> for &'a String {
    fn eq(&self, other: &Symbol) -> bool {
        self.as_str() == other.0
    }
}

impl Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// ### Represent val of attr
/// ``` #[fsm(test, trans(to="B"))] ```
/// parse into fsm{"test":Val::Empty, "trans" : Val::Map("to": Val::Str("B")}
#[derive(Debug, Clone)]
pub enum Val {
    Empty,
    Str(String),
    Map(HashMap<String, Val>),
    Vec(Vec<Val>)
}

impl Val {
    pub fn as_ident(&self) -> syn::Result<syn::Ident> {
        match self {
            Val::Str(s) => {
                let idr = syn::parse_str::<syn::Ident>(s.as_str());
                match idr {
                    Ok(ident) => { Ok(ident) }
                    Err(err) => {
                        Err(syn::Error::new(err.span(), format!("{} #Val.as_indent val={}", err.to_string(), s)))
                    }
                }
            }
            _ => { Err(syn::Error::new(Span::call_site(), format!("None Val::Str cannot convert to syn::Ident "))) }
        }
    }
    pub fn as_bin_expr(&self) -> syn::Result<syn::ExprBinary> {
        match self {
            Val::Str(s) => {
                let expr = syn::parse_str::<syn::Expr>(s.as_str())?;
                match expr {
                    Expr::Binary(bin) => {
                        Ok(bin)
                    }
                    _ => {
                        let err = syn::Error::new(Span::call_site(), format!("Only Binary Expr (eg. a+b, a*b+c) is supported"));
                        Err(err)
                    }
                }
            }
            _ => { Err(syn::Error::new(Span::call_site(), format!("None Val::Str cannot convert to syn::ExprBinary "))) }
        }
    }
    pub fn as_expr(&self) -> syn::Result<syn::Expr> {
        match self {
            Val::Str(s) => {
                let ret = syn::parse_str::<syn::Expr>(s.as_str());
                match ret {
                    Ok(expr) => { Ok(expr) }
                    Err(err) => {
                        Err(syn::Error::new(err.span(), format!("{} #Val.as_expr val={}", err.to_string(), s)))
                    }
                }
            }
            _ => { Err(syn::Error::new(Span::call_site(), format!("None Val::Str cannot convert to syn::Expr "))) }
        }
    }
}
