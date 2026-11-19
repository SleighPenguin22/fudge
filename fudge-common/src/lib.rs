#[derive(Debug, Clone, thiserror::Error)]
pub enum FudgeError<E>
where
    E: UserError,
{
    User(E),
    Internal(InternalCompilerError),
}

impl<E> From<InternalCompilerError> for FudgeError<E>
where
    E: UserError,
{
    fn from(v: InternalCompilerError) -> Self {
        Self::Internal(v)
    }
}

impl<UserErr> From<UserErr> for FudgeError<UserErr>
where
    UserErr: UserError,
{
    fn from(v: UserErr) -> Self {
        Self::User(v)
    }
}

/// A Fudge User error that requires some context to print a message
///
///
pub trait UserError: Sized {
    type ErrorContext: ?Sized;
    fn zea_error_format(&self, ctx: &Self::ErrorContext) -> FudgeErrorMessage;
    fn into_zea_error(self) -> FudgeError<Self> {
        FudgeError::User(self)
    }
}

impl UserError for std::io::Error {
    type ErrorContext = ();

    fn zea_error_format(&self, _ctx: &Self::ErrorContext) -> FudgeErrorMessage {
        let kind = self.kind();
        FudgeErrorMessage {
            msg: kind.to_string(),
            loc: None,
        }
    }
}
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub struct InternalCompilerError {
    pub stage: CompilerStage,
    pub kind: CompilerErrorKind,
}

impl std::fmt::Display for InternalCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pretty().fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
pub struct FudgeErrorMessage {
    pub msg: String,
    pub loc: Option<(usize, usize)>,
}
impl std::fmt::Display for FudgeErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(f)
    }
}
impl InternalCompilerError {
    pub const fn new(stage: CompilerStage, kind: CompilerErrorKind) -> Self {
        Self { stage, kind }
    }
    pub fn pretty(&self) -> FudgeErrorMessage {
        let s = self.stage;
        let k = &self.kind;
        let msg = format!("INTERNAL COMPILER ERROR: {s:?} : {k:?}");
        FudgeErrorMessage { msg, loc: None }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerStage {
    NonApplicable = 0,
    Parse = 1,
    ExpandInit = 2,
    TypeCheck = 3,
    LexicalScopeAnalysis = 4,
    IPRtoTHR = 5,
    CodeGen = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerErrorKind {
    IntegerOverflow,

    StrayUnscopedIdent,
    StrayPackedInit,
    GlobalNonInitStmt,
    InvalidNonGlobalSymbolUsage,
    /// A typ field is set to None after the type-checking phase
    StrayUnknownType,
    EmptyFunctionBody,
    NoEntryPoint,
}

type ZeaResult<T, E> = Result<T, FudgeError<E>>;

#[macro_export]
macro_rules! ice_bail {
    ($err:expr) => {{
        let err: InternalCompilerError = $err;
        log::error!("{}", err.pretty());
        exit(1)
    }};
}
