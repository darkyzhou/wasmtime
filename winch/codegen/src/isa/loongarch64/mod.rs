use crate::{
    abi::{wasm_sig, ABI},
    codegen::{CodeGen, CodeGenContext, FuncEnv, TypeConverter},
    frame::{DefinedLocals, Frame},
    isa::{Builder, TargetIsa},
    masm::MacroAssembler,
    regalloc::RegAlloc,
    regset::RegBitSet,
    stack::Stack,
    BuiltinFunctions,
};
use cranelift_codegen::{
    isa::loongarch64::settings as loongarch64_settings, Final, MachBufferFinalized,
    TextSectionBuilder,
};
use target_lexicon::Triple;
use wasmparser::{FuncValidator, FunctionBody, ValidatorResources};
use wasmtime_cranelift::CompiledFunction;
use wasmtime_environ::{ModuleTranslation, ModuleTypesBuilder, Tunables, VMOffsets, WasmFuncType};

mod abi;
mod address;
mod asm;
mod masm;
mod regs;

/// Create an ISA from the given triple.
pub(crate) fn isa_builder(triple: Triple) -> Builder {
    Builder::new(
        triple,
        loongarch64_settings::builder(),
        |triple, shared_flags, settings| {
            let isa_flags = loongarch64_settings::Flags::new(&shared_flags, settings);
            let isa = LoongArch64::new(triple, shared_flags, isa_flags);
            Ok(Box::new(isa))
        },
    )
}

// LoongArch64 ISA.
pub(crate) struct LoongArch64 {
    /// The target triple.
    triple: Triple,
    /// ISA specific flags.
    isa_flags: loongarch64_settings::Flags,
    /// Shared flags.
    shared_flags: Flags,
}

impl LoongArch64 {
    /// Create an LoongArch64 ISA.
    pub fn new(
        triple: Triple,
        shared_flags: Flags,
        isa_flags: loongarch64_settings::Flags,
    ) -> Self {
        Self {
            isa_flags,
            shared_flags,
            triple,
        }
    }
}

impl TargetIsa for LoongArch64 {
    fn name(&self) -> &'static str {
        "loongarch64"
    }

    fn triple(&self) -> &Triple {
        &self.triple
    }

    fn flags(&self) -> &settings::Flags {
        &self.shared_flags
    }

    fn isa_flags(&self) -> Vec<settings::Value> {
        self.isa_flags.iter().collect()
    }

    fn is_branch_protection_enabled(&self) -> bool {
        self.isa_flags.use_bti()
    }

    fn compile_function(
        &self,
        sig: &WasmFuncType,
        body: &FunctionBody,
        translation: &ModuleTranslation,
        types: &ModuleTypesBuilder,
        builtins: &mut BuiltinFunctions,
        validator: &mut FuncValidator<ValidatorResources>,
        tunables: &Tunables,
    ) -> Result<CompiledFunction> {
        todo!()
    }

    fn text_section_builder(&self, num_funcs: usize) -> Box<dyn TextSectionBuilder> {
        todo!()
    }

    fn function_alignment(&self) -> u32 {
        // See `cranelift_codegen::isa::TargetIsa::function_alignment`.
        32
    }

    fn emit_unwind_info(
        &self,
        _result: &MachBufferFinalized<Final>,
        _kind: cranelift_codegen::isa::unwind::UnwindInfoKind,
    ) -> Result<Option<cranelift_codegen::isa::unwind::UnwindInfo>> {
        // TODO: should fill this in with an actual implementation
        Ok(None)
    }

    fn page_size_align_log2(&self) -> u8 {
        todo!()
    }
}
