extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attr;
mod composited;
mod fsm;
mod internals;
mod simuples;
mod sys_dyn;
#[proc_macro_derive(Simuples, attributes(sim))]
pub fn simuples(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    simuples::expand(&mut input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
#[proc_macro_derive(Fsm, attributes(fsm))]
pub fn fsm_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut derive_input = syn::parse_macro_input!(input as syn::DeriveInput);

    fsm::expand(&mut derive_input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
#[proc_macro_derive(SysDynDerive, attributes(sim))]
pub fn dyn_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    sys_dyn::sd_gen::gen(input)
}

#[proc_macro_derive(CompositedModelDerive, attributes(sim))]
pub fn composited_model_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    composited::gen(input)
    // proc_macro::TokenStream::new()
}
