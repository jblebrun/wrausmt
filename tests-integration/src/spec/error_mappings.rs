use {
    super::{
        error::{CmdError, TestFailureError},
        runner::{CmdResult, TestResult},
    },
    wrausmt_format::{
        binary::{
            error::{BinaryParseError, BinaryParseErrorKind},
            leb128::LEB128Error,
        },
        loader::LoaderError,
        text::{
            parse::error::{ParseError, ParseErrorKind},
            resolve::ResolveError,
        },
    },
    wrausmt_runtime::{
        runtime::error::{RuntimeError, RuntimeErrorKind, TrapKind},
        validation::ValidationError,
    },
};

pub fn verify_failure<T>(result: CmdResult<T>, failure: &str) -> TestResult<()> {
    match result {
        Err(CmdError::InvocationError(re)) if matches_runtime_error(failure, &re.kind) => Ok(()),
        Err(CmdError::LoaderError(LoaderError::ParseError(ParseError { kind, .. })))
            if matches_parse_error(failure, &kind) =>
        {
            Ok(())
        }
        Err(CmdError::LoaderError(LoaderError::BinaryParseError(BinaryParseError {
            kind,
            ..
        }))) if matches_bin_parse_error(failure, &kind) => Ok(()),
        Err(CmdError::InvocationError(RuntimeError {
            kind: RuntimeErrorKind::ValidationError(ve),
            ..
        })) if matches_validation_error(failure, &ve) => Ok(()),
        Err(e) => Err(TestFailureError::failure_mismatch(failure, e)),
        _ => Err(TestFailureError::failure_missing(failure)),
    }
}

fn matches_runtime_error(failure: &str, error: &RuntimeErrorKind) -> bool {
    match error {
        RuntimeErrorKind::Trap(trap_kind) => matches_trap(failure, trap_kind),
        RuntimeErrorKind::CallStackExhaustion => failure == "call stack exhausted",
        RuntimeErrorKind::ImportMismatch(..) => failure == "incompatible import type",
        RuntimeErrorKind::ImportNotFound(..) => failure == "unknown import",
        _ => false,
    }
}

fn matches_trap(failure: &str, trap: &TrapKind) -> bool {
    match failure {
        "invalid conversion to integer" => matches!(trap, TrapKind::InvalidConversionToInteger),
        "out of bounds memory access" => {
            matches!(trap, TrapKind::OutOfBoundsMemoryAccess(..))
        }
        "out of bounds table access" => matches!(trap, TrapKind::OutOfBoundsTableAccess(..)),
        "indirect call type mismatch" => matches!(trap, TrapKind::CallIndirectTypeMismatch),
        "integer divide by zero" => matches!(trap, TrapKind::IntegerDivideByZero),
        "integer overflow" => matches!(trap, TrapKind::IntegerOverflow),
        "unreachable" => matches!(trap, TrapKind::Unreachable),
        "undefined element" => matches!(trap, TrapKind::OutOfBoundsTableAccess(..)),
        "uninitialized element" => matches!(trap, TrapKind::UninitializedElement),
        "uninitialized element 2" => matches!(trap, TrapKind::UninitializedElement),
        _ => false,
    }
}

fn matches_bin_parse_error(failure: &str, bin_parse_err: &BinaryParseErrorKind) -> bool {
    match failure {
        "integer too large" => {
            matches!(
                bin_parse_err,
                BinaryParseErrorKind::LEB128Error(LEB128Error::Overflow(_))
            ) || matches!(bin_parse_err, BinaryParseErrorKind::InvalidBoolValue(_))
        }
        // TODO - remove the need for InvalidFuncType
        "integer representation too long" => matches!(
            bin_parse_err,
            BinaryParseErrorKind::LEB128Error(LEB128Error::Unterminated(_))
                | BinaryParseErrorKind::InvalidFuncType(_)
        ),
        "magic header not detected" => {
            matches!(bin_parse_err, BinaryParseErrorKind::IncorrectMagic(_))
        }
        "unknown binary version" => {
            matches!(bin_parse_err, BinaryParseErrorKind::IncorrectVersion(_))
        }
        "malformed UTF-8 encoding" => matches!(bin_parse_err, BinaryParseErrorKind::Utf8Error(_e)),
        "malformed section id" => {
            matches!(bin_parse_err, BinaryParseErrorKind::MalformedSectionId(_))
        }
        "malformed import kind" => {
            matches!(bin_parse_err, BinaryParseErrorKind::MalformedImportKind(_))
        }
        "malformed reference type" => {
            matches!(bin_parse_err, BinaryParseErrorKind::MalformedRefType(_))
        }
        "section size mismatch" => matches!(
            bin_parse_err,
            BinaryParseErrorKind::SectionTooShort
                | BinaryParseErrorKind::SectionTooLong
                | BinaryParseErrorKind::CodeTooLong
        ),
        // TODO - remove the need for UnxpectedEndOfSectionOrFunction
        "unexpected end" => matches!(
            bin_parse_err,
            BinaryParseErrorKind::UnexpectedEnd
                | BinaryParseErrorKind::UnxpectedEndOfSectionOrFunction
        ),

        "unexpected end of section or function" | "length out of bounds" => matches!(
            bin_parse_err,
            BinaryParseErrorKind::UnxpectedEndOfSectionOrFunction
                | BinaryParseErrorKind::UnexpectedEnd
        ),
        "too many locals" => matches!(bin_parse_err, BinaryParseErrorKind::TooManyLocals),
        "END opcode expected" => matches!(bin_parse_err, BinaryParseErrorKind::CodeTooLong),
        "illegal opcode" => matches!(bin_parse_err, BinaryParseErrorKind::InvalidOpcode(_)),
        "function and code section have inconsistent lengths" => {
            matches!(bin_parse_err, BinaryParseErrorKind::FuncSizeMismatch)
        }
        "data count and data section have inconsistent lengths" => {
            matches!(bin_parse_err, BinaryParseErrorKind::DataCountMismatch)
        }
        "data count section required" => {
            matches!(bin_parse_err, BinaryParseErrorKind::DataCountMissing)
        }
        "zero byte expected" => matches!(bin_parse_err, BinaryParseErrorKind::ZeroByteExpected),
        "unexpected content after last section" => {
            matches!(
                bin_parse_err,
                BinaryParseErrorKind::UnexpectedContentAfterEnd
            )
        }
        _ => false,
    }
}

fn matches_parse_error(failure: &str, parse_err: &ParseErrorKind) -> bool {
    match failure {
        "alignment" => matches!(parse_err, ParseErrorKind::InvalidAlignment(_)),
        "i32 constant" => matches!(parse_err, ParseErrorKind::ParseIntError(_)),
        "unknown label" => matches!(parse_err, ParseErrorKind::ResolveError(_)),
        "unexpected token" => matches!(
            parse_err,
            // This should really only be unexpected token, but blocks end up parsing
            // out-of-order param/result/type as instructions. One approach to improve this
            // could be to define all of the non-instruction keywords as their own tokens.
            ParseErrorKind::UnexpectedToken(_) | ParseErrorKind::UnrecognizedInstruction(_)
        ),
        _ if failure.starts_with("unknown operator") => matches!(
            parse_err,
            // TODO - remove the need for UnexpectedToken
            ParseErrorKind::UnexpectedToken(_) | ParseErrorKind::UnrecognizedInstruction(_)
        ),
        "inline function type" => matches!(
            parse_err,
            ParseErrorKind::ResolveError(ResolveError::DuplicateTypeIndex(_))
        ),
        "malformed UTF-8 encoding" => {
            matches!(parse_err, ParseErrorKind::Utf8Error(_))
        }
        "import after function" => matches!(
            parse_err,
            ParseErrorKind::ResolveError(ResolveError::ImportAfterFunction)
        ),
        "import after global" => matches!(
            parse_err,
            ParseErrorKind::ResolveError(ResolveError::ImportAfterGlobal)
        ),
        "import after table" => matches!(
            parse_err,
            ParseErrorKind::ResolveError(ResolveError::ImportAfterTable)
        ),
        "import after memory" => matches!(
            parse_err,
            ParseErrorKind::ResolveError(ResolveError::ImportAfterMemory)
        ),
        "constant out of range" => matches!(parse_err, ParseErrorKind::InvalidNaN(_)),
        "mismatching label" => matches!(parse_err, ParseErrorKind::LabelMismatch(_, _)),
        _ => false,
    }
}

fn matches_validation_error(failure: &str, err: &ValidationError) -> bool {
    match err {
        ValidationError::ValStackUnderflow => ["type mismatch"].contains(&failure),
        ValidationError::TypeMismatch { .. } => ["type mismatch"].contains(&failure),
        _ => false,
    }
}
