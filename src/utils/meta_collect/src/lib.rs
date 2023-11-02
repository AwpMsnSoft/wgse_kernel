extern crate proc_macro;
extern crate syn;

mod collect;

use collect::{MetaCollectArgs, ToSynResult};
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

/// This macro is the 2nd part of [`WgseCommand`] enumerate member auto-fill and trait [`WgseCommandExecute`]
/// auto-implemention.
///
/// # Notes
/// This macro is ONLY used for the '`meta_collect`' feature, otherwise a compilation error will occur.
///
/// # Example
/// ```
/// #[cfg(feature = "meta_collect")]
/// {
///     use meta_collect::wgse_command;
///
///     #[wgse_command(0x00, "Nope")]
///     pub fn execute_nope(/* arguments */) -> Result<()> {
///         /* implementions */
///     }
/// }
/// ```
///
/// [`WgseCommand`]: https://just.placeholder.url
/// [`WgseCommandExecute`]: https://just.placeholder.url
#[proc_macro_attribute]
pub fn wgse_command(args: TokenStream, input: TokenStream) -> TokenStream {
    let meta_args = parse_macro_input!(args as MetaCollectArgs);
    // clone and return original ast for rust-analyzer inspection in debug mode
    let func = input.clone();
    let mut ast = parse_macro_input!(func as ItemFn);

    match collect::wgse_command_impl(meta_args, &mut ast).to_syn_result() {
        Ok(_) => (),
        Err(err) => return err.into_compile_error().into(),
    }

    #[cfg(debug_assertions)]
    {
        input
    }
    #[cfg(not(debug_assertions))]
    {
        TokenStream::new()
    }
}
