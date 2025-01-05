pub mod generated_code;

use crate::machinst::Reg;
use crate::machinst::{isle::*, CallInfo, MachInst};
use crate::machinst::{VCodeConstant, VCodeConstantData};
use crate::{
    ir::{
        immediates::*, types::*, AtomicRmwOp, BlockCall, ExternalName, Inst, InstructionData,
        MemFlags, Opcode, TrapCode, Value, ValueList,
    },
    isa::loongarch64::inst::*,
    machinst::{ArgPair, InstOutput, IsTailCall},
};
use regalloc2::PReg;
