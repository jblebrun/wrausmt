/// A macro to create a new marker type for an existing trait.
///
/// ```
/// use wrausmt_common::marker;
/// trait MarkerTrait {}
/// marker!(SubType: MarkerTrait);
/// ```
///
/// Will generate:
///
/// ```
/// trait MarkerTrait {}
/// #[derive(Clone, Copy, Debug, Default, PartialEq)]
/// pub struct SubType {}
/// impl MarkerTrait for SubType {}
/// ````
#[macro_export]
macro_rules! marker {
    (
        $(#[$($attrss:tt)*])*
        $n:ident: $t:ty
    ) => {
        $(#[$($attrss)*])*
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $n {}
        impl $t for $n {}
    };
}
