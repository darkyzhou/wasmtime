use std::u64;

use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{LibCall, TrapCode, UserExternalNameRef};
use cranelift_codegen::isa::loongarch64::inst::{self, AluOPRR, Cond, CondBrKind, ImmShift};
use cranelift_codegen::isa::loongarch64::inst::{
    emit::EmitInfo, AluOPRRI, AluOPRRR, FpuOPRR, FpuOPRRR, FpuRoundMode, Imm12, ImmShift, Inst,
};
use cranelift_codegen::isa::loongarch64::settings as loongarch64_settings;
use cranelift_codegen::{MachBuffer, Writable};

use crate::masm::{DivKind, FloatCmpKind, IntCmpKind, OperandSize, RemKind, ShiftKind};
use crate::reg::{writable, Reg, WritableReg};
use crate::CallingConvention;

use super::regs;

impl From<OperandSize> for inst::OperandSize {
    fn from(size: OperandSize) -> Self {
        match size {
            OperandSize::S32 => Self::Size32,
            OperandSize::S64 => Self::Size64,
            s => panic!("Invalid operand size {s:?}"),
        }
    }
}

impl From<IntCmpKind> for Cond {
    fn from(value: IntCmpKind) -> Self {
        match value {
            IntCmpKind::Eq => Cond::Eq,
            IntCmpKind::Ne => Cond::Ne,
            IntCmpKind::LtS => Cond::Lt,
            IntCmpKind::LtU => Cond::Ltu,
            IntCmpKind::GtS => Cond::Gt,
            IntCmpKind::GtU => Cond::Gtu,
            IntCmpKind::LeS => Cond::Le,
            IntCmpKind::LeU => Cond::Leu,
            IntCmpKind::GeS => Cond::Ge,
            IntCmpKind::GeU => Cond::Geu,
        }
    }
}

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
        // mov: Macro instruction, may need to implement by ourselves
        todo!()
    }

    /// Float register to float register move.
    pub fn fmov(&mut self, rd: WritableReg, rj: Reg, size: OperandSize) {
        // fmov
        todo!()
    }

    /// General register to float register move.
    pub fn movgr2fr(&mut self, fd: WritableReg, rj: Reg, size: OperandSize) {
        // TODO: movfr2gr
        todo!()
    }

    /// Float register to general register move.
    pub fn movfr2gr(&mut self, rd: WritableReg, fj: Reg, size: OperandSize) {
        // TODO: movfr2gr
        todo!()
    }

    /// Conditional move for general registers
    pub fn cmovgr(&mut self, dst: WritableReg, src: Reg, cc: IntCmpKind, size: OperandSize) {}

    /// Conditional move for float registers
    pub fn cmovfr(&mut self, dst: WritableReg, src: Reg, cc: IntCmpKind, size: OperandSize) {
        todo!()
    }

    /// Return instruction.
    pub fn ret(&mut self) {
        todo!()
    }

    /// Add with three registers.
    pub fn add_rrr(&mut self, rd: WritableReg, rj: Reg, rk: Reg, size: OperandSize) {
        self.emit_alu_rrr(AluOPRRR::Add, rd, rj, rk, size);
    }

    /// Add immediate and register.
    pub fn add_rri(&mut self, rd: WritableReg, rj: Reg, imm: u64, size: OperandSize) {
        let alu_op = AluOPRRI::Addi;
        if let Some(imm) = Imm12::maybe_from_u64(imm) {
            self.emit_alu_rri(alu_op, rd, rj, imm, size);
            return;
        }

        let scratch = regs::scratch();
        self.li(writable!(scratch), imm);
        self.emit_alu_rrr(alu_op, rd, rj, scratch, size);
    }

    /// Subtract with three registers.
    pub fn sub_rrr(&mut self, rd: WritableReg, rj: Reg, rk: Reg, size: OperandSize) {
        self.emit_alu_rrr(AluOPRRR::Sub, rd, rj, rk, size);
    }

    /// Subtract immediate and register.
    pub fn sub_rri(&mut self, rd: Reg, size: OperandSize) {
        let alu_op = AluOPRRI::Addi;
        if let Some(imm) = Imm12::maybe_from_u64(imm) {
            self.emit_alu_rri(alu_op, imm, rn, writable!(regs::zero()), size);
            return;
        }

        let scratch = regs::scratch();
        self.li(imm, writable!(scratch));
        self.emit_alu_rrr_extend(alu_op, scratch, rn, writable!(regs::zero()), size);
    }

    /// Multiply with three registers.
    pub fn mul_rrr(&mut self, rd: WritableReg, rj: Reg, rk: Reg, size: OperandSize) {
        self.emit_alu_rrr(AluOPRRR::Mul, rd, rj, rk, size);
    }

    /// Multiply immediate and register.
    pub fn mul_rri(&mut self, rd: WritableReg, rj: Reg, imm: u64, size: OperandSize) {
        let scratch = regs::scratch();
        self.li(writable!(scratch), imm);
        self.emit_alu_rrr(AluOPRRR::Mul, rd, rj, scratch, size);
    }

    /// Signed/unsigned division with three registers.
    pub fn div_rrr(
        &mut self,
        rd: Writable<Reg>,
        rj: Reg, // dividend
        rk: Reg, // divisor
        kind: DivKind,
        size: OperandSize,
    ) {
        // FIXME: I128 sizes

        // check for division by 0.
        self.trapz(rk, TrapCode::INTEGER_DIVISION_BY_ZERO, size);

        // check for overflow
        if kind == DivKind::Signed {
            // divisor is -1
            let scratch = regs::scratch();
            self.emit_alu_rrr(AluOPRRR::Nor, writable!(scratch), rk, regs::zero(), size);

            // dividend is minimum negative number
            let scratch2 = regs::scratch2();
            self.li(writable!(scratch2), i64::MIN as u64);
            self.emit_alu_rrr(AluOPRRR::Xor, writable!(scratch2), rj, scratch2, size);

            // combine both condition
            self.emit_alu_rrr(AluOPRRR::Or, writable!(scratch), scratch, scratch2, size);
            self.trapz(scratch, TrapCode::INTEGER_OVERFLOW, size);
        }

        let op = match Kind {
            DivKind::Signed => AluOPRRR::Div,
            DivKind::Unsigned => AluOPRRR::DivU,
        };
        self.emit_alu_rrr(op, rd.map(Into::into), rj, rk, size);
    }

    /// Signed/unsigned remainder operation with three registers.
    pub fn rem_rrr(
        &mut self,
        dest: Writable<Reg>,
        rj: Reg, // dividend
        rk: Reg, // divisor
        kind: RemKind,
        size: OperandSize,
    ) {
        // check for division by 0
        self.trapz(rk, TrapCode::INTEGER_DIVISION_BY_ZERO, size);

        let op = match Kind {
            DivKind::Signed => AluOPRRR::Mod,
            DivKind::Unsigned => AluOPRRR::ModU,
        };
        self.emit_alu_rrr(op, rd.map(Into::into), rj, rk, size);
    }

    /// And with three registers.
    pub fn and_rrr(&mut self, rd: WritableReg, rj: Reg, rk: Reg, size: OperandSize) {
        self.emit_alu_rrr(AluOPRRR::And, rd, rj, rk, size);
    }

    /// And immediate and register.
    pub fn and_rri(&mut self, rd: WritableReg, rj: Reg, imm: u64, size: OperandSize) {
        let alu_op = AluOPRRI::Andi;
        if let Some(imm) = Imm12::maybe_from_u64(imm) {
            self.emit_alu_rri(alu_op, rd, rj, imm, size);
            return;
        }

        // TODO: What will happen if we and an 32-bit imm? What will the higher 32-bit be like in scratch?
        let scratch = regs::scratch();
        self.li(writable!(scratch), imm);
        self.emit_alu_rrr(alu_op, rd, rj, scratch, size);
    }

    /// Or with three registers.
    pub fn or_rrr(&mut self, rd: WritableReg, rj: Reg, rk: Reg, size: OperandSize) {
        self.emit_alu_rrr(AluOPRRR::Or, rd, rj, rk, size);
    }

    /// Or immediate and register.
    pub fn or_rri(&mut self, rd: WritableReg, rj: Reg, imm: u64, size: OperandSize) {
        let alu_op = AluOPRRI::Ori;
        if let Some(imm) = Imm12::maybe_from_u64(imm) {
            self.emit_alu_rri(alu_op, rd, rj, imm, size);
            return;
        }

        // TODO: What will happen if we and an 32-bit imm? What will the higher 32-bit be like in scratch?
        let scratch = regs::scratch();
        self.li(writable!(scratch), imm);
        self.emit_alu_rrr(alu_op, rd, rj, scratch, size);
    }

    /// Xor with three registers.
    pub fn xor_rrr(&mut self, rd: WritableReg, rj: Reg, rk: Reg, size: OperandSize) {
        self.emit_alu_rrr(AluOPRRR::Xor, rd, rj, rk, size);
    }

    /// Xor immediate and register.
    pub fn xor_rri(&mut self, rd: WritableReg, rj: Reg, imm: u64, size: OperandSize) {
        let alu_op = AluOPRRI::Xori;
        if let Some(imm) = Imm12::maybe_from_u64(imm) {
            self.emit_alu_rri(alu_op, rd, rj, imm, size);
            return;
        }

        // TODO: What will happen if we and an 32-bit imm? What will the higher 32-bit be like in scratch?
        let scratch = regs::scratch();
        self.li(writable!(scratch), imm);
        self.emit_alu_rrr(alu_op, rd, rj, scratch, size);
    }

    /// Shift with three registers.
    pub fn shift_rrr(
        &mut self,
        rd: WritableReg,
        rj: Reg,
        rk: Reg,
        kind: ShiftKind,
        size: OperandSize,
    ) {
        let shift_op = match kind {
            ShiftKind::Shl => AluOPRRR::Sll,
            ShiftKind::ShrS => AluOPRRR::Sra,
            ShiftKind::ShrU => AluOPRRR::Srl,
            ShiftKind::Rotr => AluOPRRR::Rotr,
            ShiftKind::Rotl => {
                // TODO: Is this OK? LoongArch only counts the lowest 6-bit!
                // If kind == Rotl, then we emulate it by emitting
                // the negation of the reg rk, and returns AluOPRRR::Rotr.
                self.emit_alu_rrr(AluOPRRR::Sub, writable!(rk), regs::zero(), rk, size);
                AluOPRRR::Rotr
            }
        };
        self.emit_alu_rrr(shift_op, rd, rj, rk, size);
    }

    /// Shift immediate and register.
    pub fn shift_rri(
        &mut self,
        rd: WritableReg,
        rj: Reg,
        imm: u64,
        kind: ShiftKind,
        size: OperandSize,
    ) {
        if let Some(imm) = ImmShift::maybe_from_u64(imm) {
            let (alu_op, imm) = match kind {
                ShiftKind::Shl => (AluOPRRI::Slli, imm),
                ShiftKind::ShrS => (AluOPRRI::Srai, imm),
                ShiftKind::ShrU => (AluOPRRI::Srli, imm),
                ShiftKind::Rotr => (AluOPRRI::Rotri, imm),
                ShiftKind::Rotl => (AluOPRRI::Rotri, 64 - imm),
            };

            self.emit_alu_rri_shift(alu_op, rd, rj, imm, size);
        } else {
            // TODO: Is this possible?
            todo!("shift_rri imm unexpected range")
        }
    }

    /// Count Leading Zeros.
    pub fn clz(&mut self, rd: WritableReg, rj: Reg, size: OperandSize) {
        self.emit_alu_rr(AluOPRR::Clz, rd, rj, size);
    }

    /// Float add with three registers.
    pub fn fadd_rrr(&mut self, fd: WritableReg, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_rrr(FpuOPRRR::Add, fd, fj, fk, size);
    }

    /// Float sub with three registers.
    pub fn fsub_rrr(&mut self, fd: WritableReg, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_rrr(FpuOPRRR::Sub, fd, fj, fk, size);
    }

    /// Float mul with three registers.
    pub fn fmul_rrr(&mut self, fd: WritableReg, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_rrr(FpuOPRRR::Mul, fd, fj, fk, size);
    }

    /// Float div with three registers.
    pub fn fdiv_rrr(&mut self, fd: WritableReg, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_rrr(FpuOPRRR::Div, fd, fj, fk, size);
    }

    /// Float min with three registers.
    pub fn fmin_rrr(&mut self, fd: WritableReg, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_rrr(FpuOPRRR::Min, fd, fj, fk, size);
    }

    /// Float max with three registers.
    pub fn fmax_rrr(&mut self, fd: WritableReg, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_rrr(FpuOPRRR::Max, fd, fj, fk, size);
    }

    /// Float copysign with three registers.
    pub fn fcopysign_rrr(&mut self, fd: WritableReg, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_rrr(FpuOPRRR::CopySign, fd, fj, fk, size);
    }

    /// Float copysign with three registers.
    pub fn fcopysign_rrr(&mut self, fd: WritableReg, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_rrr(FpuOPRRR::CopySign, fd, fj, fk, size);
    }

    /// Float abs with two registers.
    pub fn fabs_rr(&mut self, fd: WritableReg, fj: Reg, size: OperandSize) {
        self.emit_fpu_rr(FpuOPRR::Abs, fd, fj, size);
    }

    /// Float neg with two registers.
    pub fn fneg_rr(&mut self, fd: WritableReg, fj: Reg, size: OperandSize) {
        self.emit_fpu_rr(FpuOPRR::Neg, fd, fj, size);
    }

    /// Float sqrt with two registers.
    pub fn fsqrt_rr(&mut self, fd: WritableReg, fj: Reg, size: OperandSize) {
        self.emit_fpu_rr(FpuOPRR::Sqrt, fd, fj, size);
    }

    /// Float round (ceil, trunc, floor) with two registers.
    pub fn fround_rr(&mut self, fd: WritableReg, fj: Reg, mode: RoundingMode, size: OperandSize) {
        let fpu_mode = match (mode, size) {
            (RoundingMode::Nearest, OperandSize::S32) => FpuRoundMode::Nearest32,
            (RoundingMode::Up, OperandSize::S32) => FpuRoundMode::Plus32,
            (RoundingMode::Down, OperandSize::S32) => FpuRoundMode::Minus32,
            (RoundingMode::Zero, OperandSize::S32) => FpuRoundMode::Zero32,
            (RoundingMode::Nearest, OperandSize::S64) => FpuRoundMode::Nearest64,
            (RoundingMode::Up, OperandSize::S64) => FpuRoundMode::Plus64,
            (RoundingMode::Down, OperandSize::S64) => FpuRoundMode::Minus64,
            (RoundingMode::Zero, OperandSize::S64) => FpuRoundMode::Zero64,
            (m, o) => panic!("Invalid rounding mode or operand size {m:?}, {o:?}"),
        };
        self.emit_fpu_round(fpu_mode, fd, fj)
    }

    /// Float compare: equal.
    pub fn fcmp_ceq(&mut self, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_cmp(FpuOPRR::Ceq, fj, fk, size);
    }

    /// Float compare: not equal.
    pub fn fcmp_cne(&mut self, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_cmp(FpuOPRR::Cne, fj, fk, size);
    }

    /// Float compare: lower than.
    pub fn fcmp_clt(&mut self, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_cmp(FpuOPRR::Clt, fj, fk, size);
    }

    /// Float compare: lower than or equal.
    pub fn fcmp_cle(&mut self, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit_fpu_cmp(FpuOPRR::Cle, fj, fk, size);
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

    /// Conditional Set sets the destination register to 1 if the condition
    /// is true, and otherwise sets it to 0.
    /// The condition is carried by $t7 (scratch2)
    pub fn cset(&mut self, rd: WritableReg, kind: IntCmpKind) {
        self.emit(Inst::CSet {
            rd,
            kind: match kind {
                IntCmpKind::Eq => IntCC::Equal,
                IntCmpKind::Ne => IntCC::NotEqual,
                IntCmpKind::LtS => IntCC::SignedLessThan,
                IntCmpKind::LtU => IntCC::UnsignedLessThan,
                IntCmpKind::GtS => IntCC::SignedGreaterThan,
                IntCmpKind::GtU => IntCC::UnsignedGreaterThan,
                IntCmpKind::LeS => IntCC::SignedLessThanOrEqual,
                IntCmpKind::LeU => IntCC::UnsignedLessThanOrEqual,
                IntCmpKind::GeS => IntCC::SignedGreaterThanOrEqual,
                IntCmpKind::GeU => IntCC::UnsignedGreaterThanOrEqual,
            },
        });
    }

    /// Float Conditional Set sets the destination register to 1
    /// if $fcc0 is true(1), and otherwise sets it to 0.
    pub fn fcset(&mut self, fd: WritableReg) {
        self.emit(Inst::FCSet { fd });
    }

    /// Trap if `rj` is zero.
    pub fn trapz(&mut self, rj: Reg, code: TrapCode, size: OperandSize) {
        self.emit(Inst::TrapIf {
            kind: CondBrKind::Zero(rj.into(), size.into()),
            trap_code: code,
        });
    }

    /// Load an immediate into a register.
    pub fn li(&mut self, rd: WritableReg, imm: u64) {
        // load_constant
        todo!()
    }

    // Helpers for ALU operations.

    fn emit_alu_rr(&mut self, op: AluOPRR, rd: WritableReg, rj: Reg, size: OperandSize) {
        self.emit(Inst::AluRR {
            alu_op: op,
            rd: rd.map(Into::into),
            rj: rj.into(),
            size,
        });
    }

    fn emit_alu_rrr(&mut self, op: AluOPRRR, rd: WritableReg, rj: Reg, rk: Reg, size: OperandSize) {
        self.emit(Inst::AluRRR {
            alu_op: op,
            rd: rd.map(Into::into),
            rj: rj.into(),
            rk: rk.into(),
            size,
        });
    }

    fn emit_alu_rri(
        &mut self,
        op: AluOPRRI,
        rd: WritableReg,
        rj: Reg,
        imm: Imm12,
        size: OperandSize,
    ) {
        self.emit(Inst::AluRRImm12 {
            alu_op: op,
            rd: rd.map(Into::into),
            rj: rj.into(),
            imm12: imm,
            size,
        });
    }

    fn emit_alu_rri_shift(
        &mut self,
        op: AluOPRRI,
        rd: WritableReg,
        rj: Reg,
        imm: ImmShift,
        size: OperandSize,
    ) {
        self.emit(Inst::AluRRImmShift {
            alu_op: op,
            rd: rd.map(Into::into),
            rj: rj.into(),
            imm,
            size,
        });
    }

    fn emit_fpu_rr(&mut self, op: FpuOPRR, fd: WritableReg, fj: Reg, size: OperandSize) {
        self.emit(Inst::FpuRR {
            alu_op: op,
            fd: fd.map(Into::into),
            fj: fj.into(),
            size,
        });
    }

    fn emit_fpu_cmp(&mut self, op: FpuOPRR, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit(Inst::FpuCmp {
            alu_op: op,
            fj: fj.into(),
            fk: fk.into(),
            size,
        });
    }

    fn emit_fpu_rrr(&mut self, op: FpuOPRRR, fd: WritableReg, fj: Reg, fk: Reg, size: OperandSize) {
        self.emit(Inst::FpuRRR {
            alu_op: op,
            fd: fd.map(Into::into),
            fj: fj.into(),
            fk: fk.into(),
            size,
        });
    }

    fn emit_fpu_round(&mut self, op: FpuRoundMode, fd: WritableReg, fj: Reg) {
        self.emit(Inst::FpuRound {
            op,
            fd: fd.map(Into::into),
            fj: fj.into(),
        });
    }

    /// Adjust the stack pointer up or down.
    pub fn sp_adjust(&mut self, amount: i32) -> SmallInstVec<Inst> {
        todo!()
    }
}
