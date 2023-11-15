use std::collections::HashMap;

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

/// Represent val of attr
/// #[fsm(trans(to="B"))]
/// into
/// ("trans" : Map("to": Str("B"))
#[derive(Debug, Clone)]
pub enum Val {
    Str(String),
    Map(HashMap<String, Val>),
}