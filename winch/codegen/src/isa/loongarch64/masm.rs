use crate::{
    abi::{self, vmctx, LocalSlot},
    codegen::ptr_type_from_ptr_size,
    masm::{CalleeKind, MacroAssembler, OperandSize, RegImm, SPOffset},
    reg::writable,
    CallingConvention,
};
use cranelift_codegen::isa::loongarch64::settings as loongarch64_settings;

use super::{abi::LoongArch64ABI, address::Address, asm::Assembler, regs};

/// LoongArch64 MacroAssembler.
pub(crate) struct MacroAssembler {
    /// Low level assembler.
    asm: Assembler,
    /// Stack pointer offset.
    sp_offset: u32,
    /// The target pointer size.
    ptr_size: OperandSize,
}

impl MacroAssembler {
    /// Create an LoongArch64 MacroAssembler.
    pub fn new(
        ptr_size: impl PtrSize,
        shared_flags: settings::Flags,
        isa_flags: loongarch64_settings::Flags,
    ) -> Self {
        Self {
            asm: Assembler::new(shared_flags, isa_flags),
            sp_offset: 0u32,
            ptr_size: ptr_type_from_ptr_size(ptr_size.size()).into(),
        }
    }
}

impl MacroAssembler {
    fn increment_sp(&mut self, bytes: u32) {
        self.sp_offset += bytes;
    }

    fn decrement_sp(&mut self, bytes: u32) {
        self.sp_offset -= bytes;
    }
}

impl Masm for MacroAssembler {
    type Address = Address;
    type Ptr = u8;
    type ABI = LoongArch64ABI;

    fn frame_setup(&mut self) {
        let ra = regs::ra();
        let fp = regs::fp();
        let sp = regs::sp();

        self.asm.sp_adjust(-32);
        self.asm
            .st(ra, Address::indexed_from_sp(24), OperandSize::S64);
        self.asm
            .st(fp, Address::indexed_from_sp(16), OperandSize::S64);
        self.asm.addi(writable!(fp), sp, 32, OperandSize::S64);
    }

    fn check_stack(&mut self, _vmctx: Reg) {
        // TODO: Implement when we have more complete assembler support.
    }

    fn frame_restore(&mut self) {
        assert_eq!(self.sp_offset, 0);

        let ra = regs::ra();
        let fp = regs::fp();

        self.asm.ld(
            writable!(ra),
            Address::indexed_from_sp(24),
            OperandSize::S64,
            true,
        );
        self.asm.ld(
            writable!(fp),
            Address::indexed_from_sp(16),
            OperandSize::S64,
            true,
        );
        self.asm.sp_adjust(32);
        self.asm.ret();
    }

    fn reserve_stack(&mut self, bytes: u32) {
        if bytes == 0 {
            return;
        }

        let sp = regs::sp();
        self.asm.sp_adjust(-bytes);
        self.increment_sp(bytes);
    }

    fn free_stack(&mut self, bytes: u32) {
        if bytes == 0 {
            return;
        }

        let sp = regs::sp();
        self.asm.sp_adjust(bytes);
        self.decrement_sp(bytes);
    }

    fn reset_stack_pointer(&mut self, offset: SPOffset) {
        self.sp_offset = offset.as_u32();
    }

    fn local_address(&mut self, local: &LocalSlot) -> Address {
        let (base, offset) = if local.addressed_from_sp() {
            // TODO: Using `expect` here is suboptimal
            let offset = self.sp_offset.checked_sub(local.offset).unwrap_or_else(|| {
                panic!(
                    "Invalid local offset = {}; sp offset = {}",
                    local.offset, self.sp_offset
                )
            });
            (regs::sp(), offset)
        } else {
            (regs::fp(), local.offset)
        };
        Address::offset(base, offset as i64)
    }

    fn address_from_sp(&self, offset: SPOffset) -> Self::Address {
        Address::indexed_from_sp((self.sp_offset - offset.as_u32()) as i64)
    }

    fn address_at_sp(&self, offset: SPOffset) -> Self::Address {
        Address::indexed_from_sp(offset.as_u32() as i64)
    }

    fn address_at_vmctx(&self, offset: u32) -> Self::Address {
        Address::offset(vmctx!(Self), offset as i64)
    }

    fn address_at_reg(&self, reg: Reg, offset: u32) -> Self::Address {
        Address::offset(reg, offset as i64)
    }

    fn call(
        &mut self,
        stack_args_size: u32,
        f: impl FnMut(&mut Self) -> (CalleeKind, CallingConvention),
    ) -> u32 {
        let alignment: u32 = <Self::ABI as abi::ABI>::call_stack_align().into();
        let addend: u32 = <Self::ABI as abi::ABI>::arg_base_offset().into();
        let alignment: u32 = <Self::ABI as abi::ABI>::call_stack_align().into();
        let addend: u32 = <Self::ABI as abi::ABI>::arg_base_offset().into();
        let delta = calculate_frame_adjustment(self.sp_offset().as_u32(), addend, alignment);
        let aligned_args_size = align_to(stack_args_size, alignment);
        let total_stack = delta + aligned_args_size;
        self.reserve_stack(total_stack);
        let (callee, call_conv) = load_callee(self);
        match callee {
            CalleeKind::Indirect(reg) => self.asm.call_with_reg(reg, call_conv),
            CalleeKind::Direct(idx) => self.asm.call_with_name(idx, call_conv),
            CalleeKind::LibCall(lib) => self.asm.call_with_lib(lib, scratch(), call_conv),
        }
        total_stack
    }

    fn sp_offset(&self) -> SPOffset {
        SPOffset::from_u32(self.sp_offset)
    }

    fn store(&mut self, src: RegImm, dst: Address, size: OperandSize) {
        let rd = match src {
            RegImm::Reg(reg) => reg,
            RegImm::Imm(imm) => {
                let imm = match imm {
                    I::I32(v) | I::F32(v) => v as u64,
                    I::F64(v) | I::I64(v) => v,
                    I::V128(_) => unreachable!(),
                };
                let scratch = regs::scratch();
                self.asm.li(writable!(scratch), imm);
                if !imm.is_float() {
                    scratch
                } else {
                    let float_scratch = regs::float_scratch();
                    self.asm.mov(writable!(float_scratch), scratch, imm.size());
                    float_scratch
                }
            }
        };

        self.asm.st(rd, dst, size);
    }

    fn add(&mut self, dst: WritableReg, lhs: Reg, rhs: RegImm, size: OperandSize) {
        match rhs {
            RegImm::Reg(rk) => {
                self.asm.add(dst, lhs, rk, size);
            }
            RegImm::Imm(v) => {
                let imm = match v {
                    I::I32(v) => v as u64,
                    I::I64(v) => v,
                    _ => todo!("add imm"),
                };
                self.asm.addi(dst, lhs, imm, size);
            }
        }
    }
}
