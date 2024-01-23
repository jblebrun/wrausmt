use {
    super::{
        error::{CmdError, TestFailureError},
        runner::{CmdResult, TestResult},
    },
    wrausmt_common::true_or::TrueOr,
    wrausmt_format::{
        binary::{
            error::{BinaryParseError, BinaryParseErrorKind},
            leb128::LEB128Error,
        },
        compiler::ValidationError,
        loader::LoaderError,
        text::{
            parse::error::{ParseError, ParseErrorKind},
            resolve::ResolveError,
        },
        ValidationErrorKind,
    },
    wrausmt_runtime::runtime::error::{RuntimeErrorKind, TrapKind},
};

pub fn verify_failure<T>(result: CmdResult<T>, failure: &str) -> TestResult<()> {
    match result {
        Err(e) => {
            matches_cmd_error(failure, &e).true_or(TestFailureError::failure_mismatch(failure, e))
        }
        Ok(_) => Err(TestFailureError::failure_missing(failure))?,
    }
}

fn matches_cmd_error(failure: &str, error: &CmdError) -> bool {
    match &error {
        CmdError::InvocationError(re) => matches_runtime_error(failure, &re.kind),
        CmdError::LoaderError(le) => matches_loader_error(failure, le),
        _ => false,
    }
}

fn matches_loader_error(failure: &str, error: &LoaderError) -> bool {
    match &error {
        LoaderError::BinaryParseError(bpe) => matches_bin_parse_error(failure, bpe),
        LoaderError::ParseError(pe) => matches_parse_error(failure, pe),
        LoaderError::ValidationError(ve) => matches_validation_error(failure, ve),
        _ => false,
    }
}

fn matches_runtime_error(failure: &str, error: &RuntimeErrorKind) -> bool {
    match error {
        RuntimeErrorKind::CallStackExhaustion => failure == "call stack exhausted",
        RuntimeErrorKind::ImportMismatch(..) => failure == "incompatible import type",
        RuntimeErrorKind::ImportNotFound(..) => failure == "unknown import",
        RuntimeErrorKind::Trap(trap_kind) => matches_trap(failure, trap_kind),
        _ => false,
    }
}

fn matches_trap(failure: &str, trap: &TrapKind) -> bool {
    match trap {
        TrapKind::CallIndirectTypeMismatch => failure == "indirect call type mismatch",
        TrapKind::IntegerDivideByZero => failure == "integer divide by zero",
        TrapKind::IntegerOverflow => failure == "integer overflow",
        TrapKind::InvalidConversionToInteger => failure == "invalid conversion to integer",
        TrapKind::OutOfBoundsMemoryAccess(..) => failure == "out of bounds memory access",
        TrapKind::OutOfBoundsTableAccess(..) => {
            ["out of bounds table access", "undefined element"].contains(&failure)
        }
        TrapKind::Unreachable => failure == "unreachable",
        TrapKind::UninitializedElement => failure.starts_with("uninitialized element"),
        _ => false,
    }
}

fn matches_bin_parse_error(failure: &str, bin_parse_err: &BinaryParseError) -> bool {
    match &bin_parse_err.kind {
        // TODO - clean up code to simplify to 1:1 mapping.
        BinaryParseErrorKind::CodeTooLong => {
            ["section size mismatch", "END opcode expected"].contains(&failure)
        }

        BinaryParseErrorKind::DataCountMismatch => {
            failure == "data count and data section have inconsistent lengths"
        }
        BinaryParseErrorKind::DataCountMissing => failure == "data count section required",
        BinaryParseErrorKind::FuncSizeMismatch => {
            failure == "function and code section have inconsistent lengths"
        }
        BinaryParseErrorKind::InvalidBoolValue(_) => [
            "integer representation too long",
            "integer too large",
            "malformed mutability",
        ]
        .contains(&failure),
        BinaryParseErrorKind::IncorrectMagic(_) => failure == "magic header not detected",
        BinaryParseErrorKind::IncorrectVersion(_) => failure == "unknown binary version",
        // TODO -- remove the need for matching InvalidFuncType
        BinaryParseErrorKind::InvalidFuncType(_) => failure == "integer represetation too long",
        BinaryParseErrorKind::InvalidOpcode(_) => failure == "illegal opcode",
        BinaryParseErrorKind::LEB128Error(le) => matches_leb_error(failure, le),
        BinaryParseErrorKind::MalformedImportKind(_) => failure == "malformed import kind",
        BinaryParseErrorKind::MalformedRefType(_) => failure == "malformed reference type",
        BinaryParseErrorKind::MalformedSectionId(_) => failure == "malformed section id",
        BinaryParseErrorKind::SectionTooLong => failure == "section size mismatch",
        BinaryParseErrorKind::SectionTooShort => failure == "section size mismatch",
        BinaryParseErrorKind::TooManyLocals => failure == "too many locals",
        BinaryParseErrorKind::UnexpectedContentAfterEnd => {
            failure == "unexpected content after last section"
        }
        BinaryParseErrorKind::UnexpectedEnd => [
            "unexpected end",
            "unexpected end of section or function",
            "length out of bounds",
        ]
        .contains(&failure),
        // TODO - remove "unexpected end" from this.
        BinaryParseErrorKind::UnxpectedEndOfSectionOrFunction => [
            "unexpected end",
            "unexpected end of section or function",
            "length out of bounds",
        ]
        .contains(&failure),
        BinaryParseErrorKind::Utf8Error(_e) => failure == "malformed UTF-8 encoding",
        BinaryParseErrorKind::ZeroByteExpected => failure == "zero byte expected",
        _ => false,
    }
}

fn matches_leb_error(failure: &str, error: &LEB128Error) -> bool {
    match &error {
        LEB128Error::Unterminated(_) => failure == "integer representation too long",
        LEB128Error::Overflow(_) => failure == "integer too large",
        _ => false,
    }
}

fn matches_parse_error(failure: &str, parse_err: &ParseError) -> bool {
    match &parse_err.kind {
        ParseErrorKind::InvalidAlignment(_) => failure == "alignment",
        ParseErrorKind::ConstantOutOfRange => [
            "i32 constant",
            "i32 constant out of range",
            "constant out of range",
        ]
        .contains(&failure),
        ParseErrorKind::ResolveError(re) => matches_resolve_error(failure, re),
        // This should really only be unexpected token, but blocks end up parsing
        // out-of-order param/result/type as instructions. One approach to improve this
        // could be to define all of the non-instruction keywords as their own tokens.
        ParseErrorKind::UnexpectedToken(_) | ParseErrorKind::UnrecognizedInstruction(_) => {
            failure == "unexpected token" || failure.starts_with("unknown operator")
        }
        ParseErrorKind::Utf8Error(_) => failure == "malformed UTF-8 encoding",

        ParseErrorKind::LabelMismatch(..) => failure == "mismatching label",
        _ => false,
    }
}

fn matches_resolve_error(failure: &str, err: &ResolveError) -> bool {
    match err {
        ResolveError::DuplicateData(_) => failure == "duplicate data",
        ResolveError::DuplicateElem(_) => failure == "duplicate elem",
        ResolveError::DuplicateFunc(_) => failure == "duplicate func",
        ResolveError::DuplicateGlobal(_) => failure == "duplicate global",
        ResolveError::DuplicateMem(_) => failure == "duplicate memory",
        ResolveError::DuplicateTable(_) => failure == "duplicate table",
        ResolveError::DuplicateTypeIndex(_) => failure == "inline function type",
        ResolveError::DuplicateLocal(_) => failure == "duplicate local",
        ResolveError::ImportAfterFunction => failure == "import after function",
        ResolveError::ImportAfterGlobal => failure == "import after global",
        ResolveError::ImportAfterMemory => failure == "import after memory",
        ResolveError::ImportAfterTable => failure == "import after table",
        ResolveError::MultipleStartSections => failure == "multiple start sections",
        ResolveError::UnresolvedLabel(_) => failure == "unknown label",
        ResolveError::UnresolvedType(_) => failure == "unknown type",
        _ => false,
    }
}

fn matches_validation_error(failure: &str, err: &ValidationError) -> bool {
    match err.kind() {
        ValidationErrorKind::AlignmentTooLarge(_) => {
            failure == "alignment must not be larger than natural"
        }
        ValidationErrorKind::BreakTypeMismatch => failure == "type mismatch",
        ValidationErrorKind::DuplicateExport => failure == "duplicate export name",
        ValidationErrorKind::ExpectedNum { .. } => failure == "type mismatch",
        ValidationErrorKind::ExpectedRef { .. } => failure == "type mismatch",
        ValidationErrorKind::InvalidConstantGlobal => failure == "unknown global",
        ValidationErrorKind::InvalidConstantInstruction => {
            failure == "constant expression required"
        }
        ValidationErrorKind::InvalidLimits => {
            failure == "size minimum must not be greater than maximum"
        }
        ValidationErrorKind::ImmutableGlobal => failure == "global is immutable",
        ValidationErrorKind::MemoryTooLarge => {
            failure == "memory size must be at most 65536 pages (4GiB)"
        }
        ValidationErrorKind::MultipleMemories => failure == "multiple memories",
        ValidationErrorKind::TypeMismatch { .. } => failure == "type mismatch",
        ValidationErrorKind::UndeclaredFunctionRef => failure == "undeclared function reference",
        ValidationErrorKind::UnknownData => failure.starts_with("unknown data segment"),
        ValidationErrorKind::UnknownElem => failure.starts_with("unknown elem segment"),
        ValidationErrorKind::UnknownFunc => failure.starts_with("unknown func"),
        ValidationErrorKind::UnknownGlobal => failure.starts_with("unknown global"),
        ValidationErrorKind::UnknownLabel => failure == "unknown label",
        ValidationErrorKind::UnknownLocal { .. } => failure == "unknown local",
        ValidationErrorKind::UnknownMemory => failure.starts_with("unknown memory"),
        ValidationErrorKind::UnknownTable => failure.starts_with("unknown table"),
        ValidationErrorKind::UnknownType => failure == "unknown type",
        ValidationErrorKind::UnsupportedSelect => failure == "invalid result arity",
        ValidationErrorKind::UnusedValues => failure == "type mismatch",
        ValidationErrorKind::ValStackUnderflow => failure == "type mismatch",
        ValidationErrorKind::WrongStartFunctionType => failure == "start function",
        ValidationErrorKind::WrongTableType => failure == "wrong table type",
        _ => false,
    }
}
