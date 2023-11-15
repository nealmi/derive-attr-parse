extern crate proc_macro;

use proc_macro::{TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use crate::parser::ctxt::Ctxt;
use crate::parser::symbol::Symbol;

mod parser;

#[proc_macro_derive(Simuples, attributes(sim))]
pub fn simuples(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    sim_expand(&mut input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Fsm, attributes(fsm))]
pub fn fsm_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut derive_input = syn::parse_macro_input!(input as syn::DeriveInput);

    fsm_expand(&mut derive_input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

const SIM: Symbol = Symbol("sim");

fn sim_expand(input: &mut syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ctx = Ctxt::new();
    let cont = parser::parse::root_from_ast(&ctx, input, SIM).unwrap().attrs;

    println!("{cont:#?}");
    ctx.check()?;

    //Do something with the info. In the case, generate System Dynamic Code.
    Ok(quote!())
}

const FSM: Symbol = Symbol("fsm");

pub(crate) fn fsm_expand(input: &mut syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ctx = Ctxt::new();
    let cont = parser::parse::root_from_ast(&ctx, input, FSM);

    println!("{cont:#?}");

    ctx.check()?;

    //Do something with the info. In the case, generate FSM Code.
    Ok(quote!())
}

