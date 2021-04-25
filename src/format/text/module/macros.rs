//! Macros providing convenient literal construction syntax for some syntax types.

/// Macro to generate an FParam instance. 
/// I32 | I64 | U32 | F32 will generate one for a numtype.
/// Func | Extern will generate one for a reftype.
#[macro_export]
macro_rules! fparam {
    ( $pid:literal; $vt:ident ) => {
        wrausmt::fparam! { Some($pid.to_string()); $vt }
    };
    ( $id:expr; Func ) => {
        wrausmt::format::text::module::syntax::FParam{ 
            id: $id,
            valuetype: wrausmt::types::RefType::Func.into() 
        }
    };
    ( $id:expr; Extern ) => {
        wrausmt::format::text::module::syntax::FParam{ 
            id: $id, 
            valuetype: wrausmt::types::RefType::Extern.into() 
        }
    };
    ( $id:expr; $vt:ident ) => {
        wrausmt::format::text::module::syntax::FParam{ 
            id: $id,
            valuetype: wrausmt::types::NumType::$vt.into()
        }
    };
    ( $vt:tt ) => {
        wrausmt::fparam! { None; $vt }
    }
}

#[macro_export]
macro_rules! fresult {
    ( $vt:ident ) => {
        wrausmt::format::text::module::syntax::FResult{ 
            valuetype: wrausmt::types::NumType::$vt.into()
        }
    }
}

#[macro_export]
macro_rules! typefield {
    ( $id:literal; [$( $pt:ident $($pid:literal)?),*] -> [$($rt:ident),*] ) => {
        typefield! { Some($id.to_string()); [$($pt $($pid)?)*] -> [$($rt)*] }
    };
    ( [$($pt:ident $($pid:literal)?),*] -> [$($rt:ident),*] ) => {
        typefield! { None; [$($pt $($pid)?)*] -> [$($rt)*] }
    };
    ( $id:expr; [$($pt:ident $($pid:literal)?),*] -> [$($rt:ident),*] ) => {
        wrausmt::format::text::module::syntax::TypeField {
            id: $id,
            params: vec![$(wrausmt::fparam! { $($pid;)? $pt })*],
            results: vec![$(wrausmt::fresult! { $rt })*],
        }
    };
}
