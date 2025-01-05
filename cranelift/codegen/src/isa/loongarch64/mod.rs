//! LoongArch 64-bit Instruction Set Architecture.

use crate::isa::loongarch64::settings as loongarch64_settings;

pub mod abi;
pub mod inst;
mod lower;
pub mod settings;
