extern crate proc_macro;

mod generate;

use generate::ToSynResult;
use proc_macro::TokenStream;

/// This macro is the 1st part of [`WgseCommand`] enumerate member auto-fill and trait [`WgseCommandExecute`]
/// auto-implemention.
///
/// # Notes
/// This macro MUST be used for the `meta_init` feature and compile before use `wgse_command`
/// and `wgse_command_trait`.
///
/// # Example
/// ```
/// #[cfg(feature = "meta_init")]
/// use meta_gen::wgse_command_interface;
///
/// #[cfg(feature = "meta_init")]
/// pub trait WgseCommandExecute {
///     #[wgse_command_interface]
///     fn execute(&self, kernel: &mut VirtualMachine, args: &BinVec<Argument>) -> Result<()>;
/// }
///
/// ```
///
/// [`WgseCommand`]: https://just.placeholder.url
/// [`WgseCommandExecute`]: https://just.placeholder.url
#[proc_macro_attribute]
pub fn wgse_command_interface(arg: TokenStream, input: TokenStream) -> TokenStream {
    match generate::wgse_command_interface_impl(arg, input).to_syn_result() {
        Ok(ast) => ast,
        Err(err) => err.into_compile_error().into(),
    }
}

/// This macro is the 3rd part of [`WgseCommand`] enumerate member auto-fill and trait [`WgseCommandExecute`]
/// auto-implemention.
///
/// # Notes
/// This macro is ONLY used without `meta_init` and `meta_collect` features.
///
/// # Example
/// ```
/// #[cfg(not(feature = "meta_collect"))]
/// {
///     use enum_dispatch::enum_dispatch;
///     #[cfg(not(feature = "meta_init"))]
///     use meta_gen::wgse_command_trait;
///
///     #[cfg(not(feature = "meta_init"))]
///     #[enum_dispatch]
///     pub trait WgseCommandExecute {
///         #[wgse_command_trait]
///         fn execute(&self, kernel: &mut VirtualMachine, args: &BinVec<Argument>) -> Result<()>;
///     }
/// }
/// ```
///
/// [`WgseCommand`]: https://just.placeholder.url
/// [`WgseCommandExecute`]: https://just.placeholder.url
#[proc_macro_attribute]
pub fn wgse_command_trait_impl(arg: TokenStream, input: TokenStream) -> TokenStream {
    match generate::wgse_command_trait_impl(arg, input).to_syn_result() {
        Ok(ast) => ast,
        Err(err) => err.into_compile_error().into(),
    }
}
