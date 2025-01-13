use std::fmt::{Display, Formatter, Result};

#[derive(Copy, Clone, Debug, Default)]
pub struct Imm12 {
    /// 16-bit container where the low 12 bits are the data payload.
    ///
    /// Acquiring the underlying value requires sign-extending the 12th bit.
    bits: u16,
}

impl Imm12 {
    pub(crate) const ZERO: Self = Self { bits: 0 };
    pub(crate) const ONE: Self = Self { bits: 1 };

    pub fn maybe_from_u64(val: u64) -> Option<Imm12> {
        Self::maybe_from_i64(val as i64)
    }

    pub fn maybe_from_i64(val: i64) -> Option<Imm12> {
        if val >= -2048 && val <= 2047 {
            Some(Imm12 {
                bits: val as u16 & 0xfff,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn from_i16(bits: i16) -> Self {
        assert!(bits >= -2048 && bits <= 2047);
        Self {
            bits: (bits & 0xfff) as u16,
        }
    }

    #[inline]
    pub fn as_i16(self) -> i16 {
        (self.bits << 4) as i16 >> 4
    }

    #[inline]
    pub fn bits(&self) -> u32 {
        self.bits.into()
    }
}

impl Into<i64> for Imm12 {
    fn into(self) -> i64 {
        self.as_i16().into()
    }
}

impl Display for Imm12 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:+}", self.as_i16())
    }
}

/// An immediate for shift instructions.
#[derive(Copy, Clone, Debug)]
pub struct ImmShift {
    /// 6-bit shift amount.
    pub imm: u8,
}

impl ImmShift {
    /// Create an ImmShift from raw bits, if possible.
    pub fn maybe_from_u64(val: u64) -> Option<ImmShift> {
        if val < 64 {
            Some(ImmShift { imm: val as u8 })
        } else {
            None
        }
    }

    /// Get the immediate value.
    pub fn value(&self) -> u8 {
        self.imm
    }
}
