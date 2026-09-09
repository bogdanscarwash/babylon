//! Honest, nominal names for persistence-layer SHA-256 values.

macro_rules! digest_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name([u8; 32]);

        impl $name {
            /// Wrap one already-computed SHA-256 value.
            #[must_use]
            pub fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            /// Return the canonical 32 bytes.
            #[must_use]
            pub fn as_bytes(&self) -> &[u8; 32] {
                &self.0
            }

            /// Render lowercase, two-digit hexadecimal without a prefix.
            #[must_use]
            pub fn to_hex(&self) -> String {
                use std::fmt::Write as _;
                self.0.iter().take(32).fold(
                    String::with_capacity(64),
                    |mut output, byte| {
                        let _ = write!(output, "{byte:02x}");
                        output
                    },
                )
            }
        }
    };
}

digest_type!(
    /// Legacy replay-lineage and idempotency stamp; it does not prove state equality.
    ReplayIdentityHash
);
digest_type!(
    /// Diagnostic hash over the canonical graph state only.
    GraphStateHash
);
digest_type!(
    /// Ordered-NUL SHA-256 identity of one migration set.
    MigrationSetDigest
);
