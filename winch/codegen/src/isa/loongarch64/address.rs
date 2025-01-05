//! LoongArch64 addressing mode.

use crate::reg::Reg;
use cranelift_codegen::ir::Constant;

/// Memory address representation.
#[derive(Debug, Copy, Clone)]
pub(crate) enum Address {
    /// Base register with an arbitrary offset.  Potentially gets
    /// lowered into multiple instructions during code emission
    /// depending on the offset.
    Offset {
        /// Base register.
        base: Reg,
        /// Offset.
        offset: i64,
    },
    /// Specialized indexed register and offset variant using
    /// the stack pointer.
    IndexedSPOffset {
        /// Offset.
        offset: i64,
    },
}

impl Address {
    /// Create register and arbitrary offset addressing mode.
    pub fn offset(base: Reg, offset: u64) -> Self {
        Self::Offset { base, offset }
    }

    /// Create an addressing mode from the stack pointer.
    pub fn indexed_from_sp(offset: i64) -> Self {
        Self::IndexedSPOffset { offset }
    }

    /// Check if the address is a made made of a base and offset.
    pub fn is_offset(&self) -> bool {
        match self {
            Self::Offset { .. } => true,
            _ => false,
        }
    }
}
