extern crate proc_macro;

mod deref;

use proc_macro::TokenStream;

/// Automatically implement [`Deref`] trait for single filed structs.
///
/// # Example
///
/// ## Single-unnamed-field tuple struct
///
/// ```
/// use derive_deref::Deref;
///
/// #[derive(Deref)]
/// struct MyInteger(i32);
///
/// let foo = MyInteger(42_i32);
/// assert_eq!(42, *foo);
/// ```
///
/// ## Single-named-field struct
///
/// ```
/// use derive_deref::Deref;
///
/// #[derive(Deref)]
/// struct MyInteger {
///     value: i32,
/// };
///
/// let foo = MyInteger { value: 42_i32 };
/// assert_eq!(42, *foo);
///
/// ```
///
/// [`Deref`]: ::std::ops::Deref
#[proc_macro_derive(Deref)]
pub fn derive_deref(input: TokenStream) -> TokenStream {
    deref::derive_deref_impl(input, false)
}

/// Automatically implement [`DerefMut`] trait for single filed struct, requires a [`Deref`]
/// implementation.
///
/// # Example
///
/// ## Single-unnamed-field tuple struct
///
/// ```
/// use derive_deref::{Deref, DerefMut};
///
/// #[derive(Deref, DerefMut)]
/// struct MyInteger(i32);
///
/// let mut foo = MyInteger(0_i32);
/// *foo = 42;
/// assert_eq!(42, *foo);
/// ```
///
/// ## Single-named-field struct
///
/// ```
/// use derive_deref::{Deref, DerefMut};
///
/// #[derive(Deref, DerefMut)]
/// struct MyInteger {
///     value: i32,
/// };
///
/// let mut foo = MyInteger { value: 0_i32 };
/// *foo = 42;
/// assert_eq!(42, *foo);
/// ```
///
/// [`DerefMut`]: ::std::ops::DerefMut
/// [`Deref`]: ::std::ops::Deref
#[proc_macro_derive(DerefMut)]
pub fn derive_deref_mut(input: TokenStream) -> TokenStream {
    deref::derive_deref_impl(input, true)
}
