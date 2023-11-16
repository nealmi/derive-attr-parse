//! # Simple parser for derive attributes
//! * "Copy from Giants". The code mostly copied from <https://github.com/serde-rs/serde/tree/master/serde_derive>.
//! * "Copy from Copier of Giants". The recommendation is to copy the code into your project for better control.<https://github.com/nealmi/derive-attr-parse>
//!
//! ### Typical usage in  proc macro
//! ```
//! let input = parse_macro_input!(inputTokenStream as DeriveInput);
//! let ctx = derive_attr_parser::Ctxt::new();
//! const DEMO: derive_attr_parser::Symbol = derive_attr_parser::Symbol("demo");
//! let container = derive_attr_parser::from_ast(&ctx, input, DEMO);
//! ```
//!### Full Demo Derive Code
//!
//! ```rust
//! extern crate proc_macro;
//!
//! use quote::quote;
//! use syn::{parse_macro_input, DeriveInput};
//! use derive_attr_parser::{Ctxt, from_ast, Symbol};
//!
//! #[proc_macro_derive(DemoDerive, attributes(demo))]
//! pub fn simuples(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//!     let mut input = parse_macro_input!(input as DeriveInput);
//!     demo_expand(&mut input)
//!         .unwrap_or_else(syn::Error::into_compile_error)
//!       .into()
//! }
//! const DEMO: Symbol = Symbol("demo");
//! fn demo_expand(input: &mut syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
//!     let ctx = Ctxt::new();
//!     let cont = from_ast(&ctx, input, DEMO);
//!     ctx.check()?;
//!     eprintln!("{cont:#?}");
//!
//!     //Do something with the info.
//!     //eg. Generate System Dynamics Code.
//!     Ok(quote!())
//! }
//! ```
//! Look into [`Container`], [`Field`], [`Val`]
//! ### Usage of Demo Derive
//! ```rust
//!#[derive(DemoDerive)]
//! #[demo(
//! name = "test",
//! method = "system_dynamics",
//! ode_solver = "eula",
//! input_name = "BassInput",
//! output_name = "BassOutput"
//! )]
//!#[demo(input(name = "BassInput", ty = "struct"))]
//!#[demo(output(name = "BassOutput", ty = "struct"))]
//! pub struct Bass {
//!     #[demo(param(val = "10_000_f64"), input(from = "total_population"))]
//!     total_population: f64,
//!     #[demo(param(val = "0.015_f64"))]
//!     ad_effectiveness: f64,
//!     #[demo(param(val = "100_f64"))]
//!     contact_rate: f64,
//!     #[demo(param(val = "0.011_f64"))]
//!     sales_fraction: f64,
//!     #[demo(var(val = "potential_clients * ad_effectiveness"))]
//!     sales_from_ad: f64,
//!     #[demo(var(
//!     val = "clients * contact_rate * sales_fraction * potential_clients / total_population"
//!     ))]
//!     sales_from_wom: f64,
//!
//!     #[demo(stock(val = "total_population"), output(to = "potential_clients"))]
//!     potential_clients: f64,
//!     #[demo(stock, output(to = "clients"))]
//!     clients: f64,
//!
//!     #[demo(
//!     flow(
//!     from = "potential_clients",
//!     to = "clients",
//!     val = "sales_from_ad + sales_from_wom"
//!     ),
//!     output(to = "sales")
//!     )]
//!     sales: f64,
//!}
//! ```
//!
mod internals;

pub use internals::ast::*;
pub use internals::ctxt::Ctxt;
pub use internals::parse::from_ast;
