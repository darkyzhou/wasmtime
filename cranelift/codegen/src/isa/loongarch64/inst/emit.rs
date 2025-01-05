//! LoongArch64 ISA: binary code emission.

use crate::isa::loongarch64::inst::*;

pub struct EmitInfo {
    shared_flag: settings::Flags,
    isa_flags: super::super::loongarch64_settings::Flags,
}

impl EmitInfo {
    pub fn new(
        shared_flag: settings::Flags,
        isa_flags: super::super::loongarch64_settings::Flags,
    ) -> Self {
        Self {
            shared_flag,
            isa_flags,
        }
    }
}
