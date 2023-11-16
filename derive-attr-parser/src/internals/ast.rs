use std::collections::HashMap;
use std::fmt::{self, Display};
use syn::{Ident, Path};

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
#[derive(Copy, Clone)]
pub struct Symbol(pub &'static str);

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
}
