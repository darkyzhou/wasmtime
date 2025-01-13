use crate::ir::types::*;
use crate::isa::loongarch64::inst::*;

/// Type used to communicate the operand size of a machine instruction,
/// as LoongArch64 has 32- and 64-bit variants of many instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperandSize {
    /// 32-bit.
    Size32,
    /// 64-bit.
    Size64,
}

impl OperandSize {
    /// 32-bit case?
    pub fn is32(self) -> bool {
        self == OperandSize::Size32
    }

    /// 64-bit case?
    pub fn is64(self) -> bool {
        self == OperandSize::Size64
    }

    /// Convert from a needed width to the smallest size that fits.
    pub fn from_bits<I: Into<usize>>(bits: I) -> OperandSize {
        let bits: usize = bits.into();
        assert!(bits <= 64);
        if bits <= 32 {
            OperandSize::Size32
        } else {
            OperandSize::Size64
        }
    }

    /// Return the operand size in bits.
    pub fn bits(&self) -> u8 {
        match self {
            OperandSize::Size32 => 32,
            OperandSize::Size64 => 64,
        }
    }

    /// Convert from an integer type into the smallest size that fits.
    pub fn from_ty(ty: Type) -> OperandSize {
        debug_assert!(!ty.is_vector());

        Self::from_bits(ty_bits(ty))
    }

    /// Convert to I32, I64, or I128.
    pub fn to_ty(self) -> Type {
        match self {
            OperandSize::Size32 => I32,
            OperandSize::Size64 => I64,
        }
    }

    /// Register interpretation bit.
    /// When 0, the register is interpreted as the 32-bit version.
    /// When 1, the register is interpreted as the 64-bit version.
    pub fn sf_bit(&self) -> u32 {
        match self {
            OperandSize::Size32 => 0,
            OperandSize::Size64 => 1,
        }
    }

    /// The maximum unsigned value representable in a value of this size.
    pub fn max_value(&self) -> u64 {
        match self {
            OperandSize::Size32 => u32::MAX as u64,
            OperandSize::Size64 => u64::MAX,
        }
    }
}

/// Condition for conditional branches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cond {
    /// Equal.
    Eq,
    /// Not equal.
    Ne,
    /// Signed less than.
    Lt,
    /// Signed less than or equal to.
    Le,
    /// Signed greater than.
    Gt,
    /// Signed greater than or equal to.
    Ge,
    /// Unsigned less than.
    Ltu,
    /// Unsigned less than or equal to.
    Leu,
    /// Unsigned greater than.
    Gtu,
    /// Unsigned greater than or equal to.
    Geu,
}

impl Cond {
    /// Return the inverted condition.
    pub fn invert(self) -> Cond {
        match self {
            Cond::Eq => Cond::Ne,
            Cond::Ne => Cond::Eq,
            Cond::Lt => Cond::Ge,
            Cond::Le => Cond::Gt,
            Cond::Gt => Cond::Le,
            Cond::Ge => Cond::Lt,
            Cond::Ltu => Cond::Geu,
            Cond::Leu => Cond::Gtu,
            Cond::Gtu => Cond::Leu,
            Cond::Geu => Cond::Ltu,
        }
    }
}

/// The kind of conditional branch
#[derive(Clone, Copy, Debug)]
pub enum CondBrKind {
    /// Condition: given register is zero.
    Zero(Reg, OperandSize),
    /// Condition: given register is nonzero.
    NotZero(Reg, OperandSize),
    /// Condition: the given condition-code test is true.
    Cond(Cond),
}

impl CondBrKind {
    /// Return the inverted branch condition.
    pub fn invert(self) -> CondBrKind {
        match self {
            CondBrKind::Zero(reg, size) => CondBrKind::NotZero(reg, size),
            CondBrKind::NotZero(reg, size) => CondBrKind::Zero(reg, size),
            CondBrKind::Cond(cond) => CondBrKind::Cond(cond.invert()),
        }
    }
}
