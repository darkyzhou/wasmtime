use cranelift_codegen::ir::{LibCall, UserExternalNameRef};
use cranelift_codegen::isa::loongarch64::inst::Imm12;
use cranelift_codegen::isa::loongarch64::settings as loongarch64_settings;
use cranelift_codegen::{isa::loongarch64::inst::emit::EmitInfo, MachBuffer};

use crate::masm::OperandSize;
use crate::reg::{Reg, WritableReg};
use crate::CallingConvention;

/// Low level assembler implementation for LoongArch64.
pub(crate) struct Assembler {
    /// The machine instruction buffer.
    buffer: MachBuffer<Inst>,
    /// Constant emission information.
    emit_info: EmitInfo,
    /// Emission state.
    emit_state: EmitState,
}

impl Assembler {
    /// Create a new LoongArch64 assembler.
    pub fn new(shared_flags: settings::Flags, isa_flags: loongarch64_settings::Flags) -> Self {
        Self {
            buffer: MachBuffer::<Inst>::new(),
            emit_state: Default::default(),
            emit_info: EmitInfo::new(shared_flags, isa_flags),
        }
    }

    /// Load a register.
    pub fn ld(&mut self, rd: Reg, addr: Address, size: OperandSize, signed: bool) {
        todo!()
    }

    /// Store a register.
    pub fn st(&mut self, rd: Reg, addr: Address, size: OperandSize) {
        todo!()
    }

    /// Register to register move.
    pub fn mov(&mut self, rd: WritableReg, rj: Reg, size: OperandSize) {
        todo!()
    }

    /// Return instruction.
    pub fn ret(&mut self) {
        todo!()
    }

    /// Add with three registers.
    pub fn add(&mut self, rd: WritableReg, rj: Reg, rk: Reg, size: OperandSize) {
        todo!()
    }

    /// Subtract with three registers.
    pub fn sub(&mut self, rd: WritableReg, rj: Reg, rk: Reg, size: OperandSize) {
        todo!()
    }

    /// Add immediate and register.
    pub fn addi(&mut self, rd: WritableReg, rj: Reg, imm: u64, size: OperandSize) {
        /// First check if fits imm12, then imm16, then catch all
        todo!()
    }

    /// Load an immediate into a register.
    pub fn li(&mut self, rd: WritableReg, imm: u64) {
        // load_constant
        todo!()
    }

    /// Emit a direct call to a function defined locally and
    /// referenced to by `name`.
    pub fn call_with_name(&mut self, name: UserExternalNameRef, call_conv: CallingConvention) {
        // self.emit(Inst::Call {
        //     info: Box::new(cranelift_codegen::CallInfo::empty(
        //         ExternalName::user(name),
        //         call_conv.into(),
        //     )),
        // })
        todo!()
    }

    /// Emit an indirect call to a function whose address is
    /// stored the `callee` register.
    pub fn call_with_reg(&mut self, callee: Reg, call_conv: CallingConvention) {
        // self.emit(Inst::CallInd {
        //     info: Box::new(cranelift_codegen::CallInfo::empty(
        //         callee.into(),
        //         call_conv.into(),
        //     )),
        // })
        todo!()
    }

    /// Emit a call to a well-known libcall.
    /// `dst` is used as a scratch register to hold the address of the libcall function.
    pub fn call_with_lib(&mut self, lib: LibCall, dst: Reg, call_conv: CallingConvention) {
        // let name = ExternalName::LibCall(lib);
        // self.emit(Inst::LoadExtName {
        //     rd: writable!(dst.into()),
        //     name: name.into(),
        //     offset: 0,
        // });
        // self.call_with_reg(dst, call_conv)
        todo!()
    }

    /// Adjust the stack pointer up or down.
    pub fn sp_adjust(&mut self, amount: i32) -> SmallInstVec<Inst> {
        todo!()
    }
}
