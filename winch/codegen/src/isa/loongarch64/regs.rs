//! LoongArch64 register definition.
//! https://loongson.github.io/LoongArch-Documentation/LoongArch-ELF-ABI-EN.html#_register_convention

use crate::isa::reg::Reg;
use regalloc2::{PReg, RegClass};

/// GPR index bound.
pub(crate) const MAX_GPR: u32 = 32;
/// FPR index bound.
pub(crate) const MAX_FPR: u32 = 32;

/// Construct a R-register from an index.
pub(crate) const fn rreg(num: u8) -> Reg {
    assert!((num as u32) < MAX_GPR);
    Reg::new(PReg::new(num as usize, RegClass::Int))
}

/// Construct a F-register from an index.
pub(crate) const fn freg(num: u8) -> Reg {
    assert!((num as u32) < MAX_FPR);
    Reg::new(PReg::new(num as usize, RegClass::Float))
}

/// Scratch register.
pub(crate) const fn scratch() -> Reg {
    rreg(20)
}

// Float scratch register.
pub(crate) const fn float_scratch() -> Reg {
    freg(22)
}

// Alias to return address register
pub(crate) const fn ra() -> Reg {
    rreg(1)
}

// Alias to frame pointer
pub(crate) const fn fp() -> Reg {
    rreg(22)
}

// Alias to stack pointer
pub(crate) const fn sp() -> Reg {
    rreg(3)
}
