use crate::{
    abi::{self, vmctx, LocalSlot},
    codegen::{ptr_type_from_ptr_size, CodeGenContext, Emission},
    isa::{
        reg::{writable, Reg, WritableReg},
        CallingConvention,
    },
    masm::{
        CalleeKind, DivKind, ExtendKind, FloatCmpKind, Imm, IntCmpKind, MacroAssembler as Masm,
        OperandSize, RegImm, RemKind, SPOffset,
    },
    stack::TypedReg,
    CallingConvention,
};
use cranelift_codegen::{ir::TrapCode, isa::loongarch64::settings as loongarch64_settings};
use regalloc2::RegClass;
use wasmtime_environ::WasmValType;

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
            writable!(fp),
            Address::indexed_from_sp(16),
            OperandSize::S64,
            true,
        );
        self.asm.ld(
            writable!(ra),
            Address::indexed_from_sp(24),
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
                    self.asm
                        .movgr2fr(writable!(float_scratch), scratch, imm.size());
                    float_scratch
                }
            }
        };

        self.asm.st(rd, dst, size);
    }

    fn store_ptr(&mut self, src: Reg, dst: Self::Address) {
        self.store(src.into(), dst, self.ptr_size);
    }

    fn wasm_store(&mut self, src: Reg, dst: Self::Address, size: OperandSize) {
        // TODO: untrusted
        self.asm.st(src, dst, size);
    }

    fn load(&mut self, src: Self::Address, dst: WritableReg, size: OperandSize) {
        self.asm.ld(dst, src, size, false);
    }

    fn wasm_load(
        &mut self,
        src: Self::Address,
        dst: WritableReg,
        size: OperandSize,
        kind: Option<ExtendKind>,
    ) {
        // kind is some if the value is signed
        // unlike x64, unused bits are set to zero so we don't need to extend
        // FIXME: verify this
        self.asm.ld(dst, src, size, kind.is_some());
    }

    fn load_ptr(&mut self, src: Self::Address, dst: WritableReg) {
        self.load(dst, src, self.ptr_size);
    }

    fn load_addr(&mut self, src: Self::Address, dst: WritableReg, size: OperandSize) {
        self.asm.ld(dst, src, size, false);
    }

    fn pop(&mut self, dst: WritableReg, size: OperandSize) {
        let addr = self.address_from_sp(SPOffset::from_u32(self.sp_offset));
        self.asm.ld(dst, addr, size, false);
        self.free_stack(size.bytes());
    }

    fn mov(&mut self, dst: WritableReg, src: RegImm, size: OperandSize) {
        match src {
            RegImm::Reg(src) => match (dst.to_reg().class(), src.class()) {
                (RegClass::Int, RegClass::Int) => self.asm.mov(dst, src, size),
                (RegClass::Float, RegClass::Float) => self.asm.fmov(dst, src, size),
                (RegClass::Float, RegClass::Int) => self.asm.movgr2fr(dst, src, size),
                // TODO: Float -> Int?
                _ => todo!("mov float->int"),
            },
            RegImm::Imm(imm) => {
                let imm = match imm {
                    I::I32(v) | I::F32(v) => v as u64,
                    I::F64(v) | I::I64(v) => v,
                    I::V128(_) => todo!("mov imm type"),
                };
                let scratch = regs::scratch();
                self.asm.li(writable!(scratch), imm);
                match dst.to_reg().class() {
                    RegClass::Int => self.asm.mov(dst, scratch, size),
                    RegClass::Float => self.asm.movgr2fr(dst, scratch, size),
                    _ => todo!("mov dst class"),
                }
            }
        }
    }

    fn cmov(&mut self, dst: WritableReg, src: Reg, cc: IntCmpKind, size: OperandSize) {
        match (dst.to_reg().class(), src.class()) {
            (RegClass::Int, RegClass::Int) => self.asm.cmovgr(dst, src, cc, size),
            (RegClass::Float, RegClass::Float) => self.asm.cmovfr(dst, src, cc, size),
            _ => todo!("cmov float int"),
        }
    }

    fn add(&mut self, dst: WritableReg, lhs: Reg, rhs: RegImm, size: OperandSize) {
        match rhs {
            RegImm::Reg(rk) => {
                self.asm.add_rrr(dst, lhs, rk, size);
            }
            RegImm::Imm(imm) => {
                let imm = match v {
                    Imm::I32(v) => v as u64,
                    Imm::I64(v) => v,
                    _ => todo!("add imm"),
                };

                self.asm.add_rri(dst, lhs, imm, size);
            }
        }
    }

    fn checked_uadd(
        &mut self,
        dst: WritableReg,
        lhs: Reg,
        rhs: RegImm,
        size: OperandSize,
        trap: TrapCode,
    ) {
        // TODO: TrapIf
    }

    fn sub(&mut self, dst: WritableReg, lhs: Reg, rhs: RegImm, size: OperandSize) {
        match rhs {
            RegImm::Reg(rk) => {
                self.asm.sub_rrr(dst, lhs, rk, size);
            }
            RegImm::Imm(imm) => {
                let imm = match imm {
                    Imm::I32(v) => (-v) as u64,
                    Imm::I64(v) => (-v) as u64,
                    _ => todo!("add imm"),
                };

                self.asm.add_rri(dst, lhs, imm, size);
            }
        }
    }

    fn mul(&mut self, dst: WritableReg, lhs: Reg, rhs: RegImm, size: OperandSize) {
        match rhs {
            RegImm::Reg(rk) => {
                self.asm.mul_rrr(dst, lhs, rk, size);
            }
            RegImm::Imm(imm) => {
                let imm = match imm {
                    Imm::I32(v) => v as u64,
                    Imm::I64(v) => v,
                    _ => todo!("mul imm"),
                };

                self.asm.mul_rri(dst, lhs, imm, size);
            }
        }
    }

    fn float_add(&mut self, dst: WritableReg, lhs: Reg, rhs: Reg, size: OperandSize) {
        self.asm.fadd_rrr(dst, lhs, rhs, size);
    }

    fn float_sub(&mut self, dst: WritableReg, lhs: Reg, rhs: Reg, size: OperandSize) {
        self.asm.fsub_rrr(dst, lhs, rhs, size);
    }

    fn float_mul(&mut self, dst: WritableReg, lhs: Reg, rhs: Reg, size: OperandSize) {
        self.asm.fmul_rrr(dst, lhs, rhs, size);
    }

    fn float_div(&mut self, dst: WritableReg, lhs: Reg, rhs: Reg, size: OperandSize) {
        self.asm.fdiv_rrr(dst, lhs, rhs, size);
    }

    fn float_min(&mut self, dst: WritableReg, lhs: Reg, rhs: Reg, size: OperandSize) {
        self.asm.fmin_rrr(dst, lhs, rhs, size);
    }

    fn float_max(&mut self, dst: WritableReg, lhs: Reg, rhs: Reg, size: OperandSize) {
        self.asm.fmax_rrr(dst, lhs, rhs, size);
    }

    fn float_copysign(&mut self, dst: WritableReg, lhs: Reg, rhs: Reg, size: OperandSize) {
        self.asm.fcopysign_rrr(dst, lhs, rhs, size);
    }

    fn float_abs(&mut self, dst: WritableReg, size: OperandSize) {
        self.asm.fabs_rr(dst, dst.to_reg(), size);
    }

    fn float_neg(&mut self, dst: WritableReg, size: OperandSize) {
        self.asm.fneg_rr(dst, dst.to_reg(), size);
    }

    fn float_round<F: FnMut(&mut FuncEnv<Self::Ptr>, &mut CodeGenContext<Emission>, &mut Self)>(
        &mut self,
        mode: RoundingMode,
        _env: &mut FuncEnv<Self::Ptr>,
        context: &mut CodeGenContext<Emission>,
        size: OperandSize,
        _fallback: F,
    ) {
        let src = context.pop_to_reg(self, None);
        self.asm
            .fround_rr(writable!(src.into()), src.into(), mode, size);
        context.stack.push(src.into());
    }

    fn float_sqrt(&mut self, dst: WritableReg, src: Reg, size: OperandSize) {
        self.asm.fsqrt_rr(dst, src, size);
    }

    fn and(&mut self, dst: WritableReg, lhs: Reg, rhs: RegImm, size: OperandSize) {
        match rhs {
            RegImm::Reg(rk) => {
                self.asm.and_rrr(dst, lhs, rk, size);
            }
            RegImm::Imm(imm) => {
                let imm = match imm {
                    I::I32(v) => v as u64,
                    I::I64(v) => v,
                    _ => todo!("and imm"),
                };

                self.asm.and_rri(dst, lhs, imm, size);
            }
        }
    }

    fn or(&mut self, dst: WritableReg, lhs: Reg, rhs: RegImm, size: OperandSize) {
        match rhs {
            RegImm::Reg(rk) => {
                self.asm.or_rrr(dst, lhs, rk, size);
            }
            RegImm::Imm(imm) => {
                let imm = match imm {
                    I::I32(v) => v as u64,
                    I::I64(v) => v,
                    _ => todo!("or imm"),
                };

                self.asm.or_rri(dst, lhs, imm, size);
            }
        }
    }

    fn xor(&mut self, dst: WritableReg, lhs: Reg, rhs: RegImm, size: OperandSize) {
        match rhs {
            RegImm::Reg(rk) => {
                self.asm.xor_rrr(dst, lhs, rk, size);
            }
            RegImm::Imm(imm) => {
                let imm = match imm {
                    I::I32(v) => v as u64,
                    I::I64(v) => v,
                    _ => todo!("xor imm"),
                };

                self.asm.xor_rri(dst, lhs, imm, size);
            }
        }
    }

    /// Perform a shift operation between a register and an immediate.
    fn shift_ir(
        &mut self,
        dst: WritableReg,
        imm: u64,
        lhs: Reg,
        kind: ShiftKind,
        size: OperandSize,
    ) {
        self.asm.shift_rri(dst, lhs, imm, kind, size);
    }

    fn shift(
        &mut self,
        context: &mut CodeGenContext<Emission>,
        kind: ShiftKind,
        size: OperandSize,
    ) {
        let src = context.pop_to_reg(self, None);
        let dst = context.pop_to_reg(self, None);

        self.asm
            .shift_rrr(writable!(dst.into()), dst.into(), src.into(), kind, size);

        context.free_reg(src);
        context.stack.push(dst.into());
    }

    fn div(&mut self, context: &mut CodeGenContext<Emission>, kind: DivKind, size: OperandSize) {
        context.binop(self, size, |this, dividend, divisor, size| {
            this.asm
                .div_rrr(writable!(dividend), dividend, divisor, kind, size);
            match size {
                OperandSize::S32 => TypedReg::new(WasmValType::I32, dividend),
                OperandSize::S64 => TypedReg::new(WasmValType::I64, dividend),
                s => unreachable!("invalid size for division: {s:?}"),
            }
        })
    }

    fn rem(&mut self, context: &mut CodeGenContext<Emission>, kind: RemKind, size: OperandSize) {
        context.binop(self, size, |this, dividend, divisor, size| {
            this.asm
                .rem_rrr(writable!(dividend), dividend, divisor, kind, size);
            match size {
                OperandSize::S32 => TypedReg::new(WasmValType::I32, dividend),
                OperandSize::S64 => TypedReg::new(WasmValType::I64, dividend),
                s => unreachable!("invalid size for remainder: {s:?}"),
            }
        })
    }

    fn cmp(&mut self, src1: Reg, src2: RegImm, size: OperandSize) {
        let scratch = regs::scratch2();
        match src2 {
            RegImm::Reg(rk) => {
                self.asm.sub_rrr(writable!(scratch), src1, rk, size);
            }
            RegImm::Imm(imm) => {
                let imm = match v {
                    Imm::I32(v) => (-v) as u64,
                    Imm::I64(v) => (-v) as u64,
                    _ => todo!("cmp imm"),
                };

                self.asm.add_rri(writable!(scratch), src1, imm, size);
            }
        }
    }

    fn cmp_with_set(&mut self, dst: WritableReg, src: RegImm, kind: IntCmpKind, size: OperandSize) {
        let scratch = regs::scratch2();
        match src {
            RegImm::Reg(rk) => {
                self.asm.sub_rrr(writable!(scratch), dst.to_reg(), rk, size);
            }
            RegImm::Imm(imm) => {
                let imm = match v {
                    Imm::I32(v) => (-v) as u64,
                    Imm::I64(v) => (-v) as u64,
                    _ => todo!("cmp_with_set imm"),
                };

                self.asm
                    .add_rri(writable!(scratch), dst.to_reg(), imm, size);
            }
        }
        self.asm.cset(dst, kind.into());
    }

    fn float_cmp_with_set(
        &mut self,
        dst: WritableReg,
        src1: Reg,
        src2: Reg,
        kind: FloatCmpKind,
        size: OperandSize,
    ) {
        match kind {
            FloatCmpKind::Eq => self.asm.fcmp_ceq(src1, src2, size),
            FloatCmpKind::Ne => self.asm.fcmp_cne(src1, src2, size),
            FloatCmpKind::Lt => self.asm.fcmp_clt(src1, src2, size),
            FloatCmpKind::Gt => self.asm.fcmp_clt(src2, src1, size),
            FloatCmpKind::Le => self.asm.fcmp_cle(src1, src2, size),
            FloatCmpKind::Ge => self.asm.fcmp_cle(src2, src1, size),
        }
        self.asm.fcset(dst);
    }

    fn clz(&mut self, dst: WritableReg, src: Reg, size: OperandSize) {
        self.asm.clz(dst, src, size);
    }

    fn trapif(&mut self, cc: IntCmpKind, code: TrapCode) {}
}
