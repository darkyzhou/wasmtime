use super::regs;
use crate::abi::{align_to, ABIOperand, ABIParams, ABIResults, ABISig, ParamsOrReturns, ABI};
use crate::isa::{reg::Reg, CallingConvention};
use crate::RegIndexEnv;
use wasmtime_environ::{WasmHeapType, WasmRefType, WasmValType};

#[derive(Default)]
pub(crate) struct LoongArch64ABI;

impl ABI for LoongArch64ABI {
    fn stack_align() -> u8 {
        16
    }

    // TODO
    fn call_stack_align() -> u8 {
        16
    }

    // TODO
    fn arg_base_offset() -> u8 {
        16
    }

    fn word_bits() -> u8 {
        64
    }

    fn sig_from(
        params: &[WasmValType],
        returns: &[WasmValType],
        call_conv: &CallingConvention,
    ) -> ABISig {
        todo!()
    }

    fn abi_results(returns: &[WasmValType], call_conv: &CallingConvention) -> ABIResults {
        todo!()
    }

    fn scratch_for(ty: &WasmValType) -> Reg {
        todo!()
    }

    fn vmctx_reg() -> Reg {
        todo!()
    }

    fn stack_slot_size() -> u8 {
        Self::word_bytes()
    }

    fn sizeof(ty: &WasmValType) -> u8 {
        todo!()
    }
}
