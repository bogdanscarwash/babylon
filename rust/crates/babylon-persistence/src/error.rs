//! Failure categories at the engine-to-PostgreSQL boundary.

/// The five failure stages callers must handle separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceFailureKind {
    /// A bounded connection operation failed.
    Connection,
    /// Schema adoption or migration failed.
    Migration,
    /// A typed row or envelope could not be serialized.
    Serialization,
    /// `PostgreSQL` rejected a declared invariant.
    Constraint,
    /// The final transaction commit failed.
    Commit,
}

/// One persistence failure with no generic catch-all variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceError {
    /// Connection-stage detail.
    Connection(Box<str>),
    /// Migration-stage detail.
    Migration(Box<str>),
    /// Serialization-stage detail.
    Serialization(Box<str>),
    /// Constraint-stage detail.
    Constraint(Box<str>),
    /// Commit-stage detail.
    Commit(Box<str>),
}

impl PersistenceError {
    /// Construct a connection failure.
    pub fn connection(detail: impl Into<Box<str>>) -> Self {
        Self::Connection(detail.into())
    }

    /// Construct a migration failure.
    pub fn migration(detail: impl Into<Box<str>>) -> Self {
        Self::Migration(detail.into())
    }

    /// Construct a serialization failure.
    pub fn serialization(detail: impl Into<Box<str>>) -> Self {
        Self::Serialization(detail.into())
    }

    /// Construct a constraint failure.
    pub fn constraint(detail: impl Into<Box<str>>) -> Self {
        Self::Constraint(detail.into())
    }

    /// Construct a commit failure.
    pub fn commit(detail: impl Into<Box<str>>) -> Self {
        Self::Commit(detail.into())
    }

    /// Return the stage without discarding its detail.
    #[must_use]
    pub fn kind(&self) -> PersistenceFailureKind {
        match self {
            Self::Connection(_) => PersistenceFailureKind::Connection,
            Self::Migration(_) => PersistenceFailureKind::Migration,
            Self::Serialization(_) => PersistenceFailureKind::Serialization,
            Self::Constraint(_) => PersistenceFailureKind::Constraint,
            Self::Commit(_) => PersistenceFailureKind::Commit,
        }
    }

    fn detail(&self) -> &str {
        match self {
            Self::Connection(detail)
            | Self::Migration(detail)
            | Self::Serialization(detail)
            | Self::Constraint(detail)
            | Self::Commit(detail) => detail,
        }
    }
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?} failure: {}", self.kind(), self.detail())
    }
}

impl std::error::Error for PersistenceError {}
