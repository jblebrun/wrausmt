//! Macros providing convenient literal construction syntax for some syntax
//! types.

/// Macro to generate an FParam instance.
/// I32 | I64 | U32 | F32 will generate one for a numtype.
/// Func | Extern will generate one for a reftype.
#[macro_export]
macro_rules! fparam {
    ( $pid:literal; $vt:ident ) => {
        wrausmt_runtime::fparam! { Some($pid.into()); $vt }
    };
    ( $id:expr; Func ) => {
        wrausmt_runtime::syntax::FParam {
            id:        $id,
            valuetype: wrausmt_runtime::syntax::types::RefType::Func.into(),
        }
    };
    ( $id:expr; Extern ) => {
        wrausmt_runtime::syntax::FParam {
            id:        $id,
            valuetype: wrausmt_runtime::syntax::types::RefType::Extern.into(),
        }
    };
    ( $id:expr; $vt:ident ) => {
        wrausmt_runtime::syntax::FParam {
            id:        $id,
            valuetype: wrausmt_runtime::syntax::types::NumType::$vt.into(),
        }
    };
    ( $vt:tt ) => {
        wrausmt_runtime::fparam! { None; $vt }
    };
}

#[macro_export]
macro_rules! fresult {
    ( $vt:ident ) => {
        wrausmt_runtime::syntax::FResult {
            valuetype: wrausmt_runtime::syntax::types::NumType::$vt.into(),
        }
    };
}

#[macro_export]
macro_rules! typefield {
    ( $id:literal; [$( $pt:ident $($pid:literal)?),*] -> [$($rt:ident),*] ) => {
        typefield! { Some($id.into()); [$($pt $($pid.into())?)*] -> [$($rt)*] }
    };
    ( [$($pt:ident $($pid:literal)?),*] -> [$($rt:ident),*] ) => {
        typefield! { None; [$($pt $($pid)?)*] -> [$($rt)*] }
    };
    ( $id:expr; [$($pt:ident $($pid:literal)?),*] -> [$($rt:ident),*] ) => {
        wrausmt_runtime::syntax::TypeField {
            id: $id,
            functiontype: $crate::syntax::FunctionType {
                params: vec![$(wrausmt_runtime::fparam! { $($pid;)? $pt })*],
                results: vec![$(wrausmt_runtime::fresult! { $rt })*],
            }
        }
    };
}
