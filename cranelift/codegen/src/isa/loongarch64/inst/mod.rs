//! This module defines loongarch64-specific machine instruction types.

pub use crate::ir::condcodes::{FloatCC, IntCC};
pub use crate::isa::loongarch64::lower::isle::generated_code::{
    AMode, AluOPRRI, AluOPRRR, AluOPRR, FpuOPRR, FpuOPRRR, FpuRoundMode, LoadOP, MInst as Inst,
    StoreOP,
};
use crate::machinst::*;
use crate::{settings, CodegenError, CodegenResult};

pub mod args;
pub use self::args::*;
pub mod emit;
pub use self::emit::*;
pub mod imms;
pub use self::imms::*;
