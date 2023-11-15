use syn::DeriveInput;
use crate::internals::ctxt::Ctxt;
use crate::internals::symbol::Symbol;

mod ctxt;
mod parser;
mod symbol;

pub const SIM: Symbol = Symbol("sim");

