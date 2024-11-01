use crate::modeling::bmc::context::BMCJingleContext;
use crate::modeling::bmc::machine::cpu::{ConcretePcodeAddress, PcodeMachineAddress};
use crate::modeling::bmc::machine::memory::MemoryState;
use crate::JingleError;
use cpu::SymbolicPcodeAddress;
use jingle_sleigh::PcodeOperation;

pub(crate) mod cpu;
pub(crate) mod memory;
pub struct MachineState<'ctx, 'sl> {
    jingle: BMCJingleContext<'ctx, 'sl>,
    memory: MemoryState<'ctx, 'sl>,
    pc: SymbolicPcodeAddress<'ctx>,
}

impl<'ctx, 'sl> MachineState<'ctx, 'sl> {
    pub fn fresh(jingle: &BMCJingleContext<'ctx, 'sl>) -> Self {
        Self {
            jingle: jingle.clone(),
            memory: MemoryState::fresh(jingle),
            pc: SymbolicPcodeAddress::fresh(jingle.z3),
        }
    }

    pub fn fresh_for_address(
        jingle: &BMCJingleContext<'ctx, 'sl>,
        machine_addr: PcodeMachineAddress,
    ) -> Self {
        let pc = ConcretePcodeAddress::from(machine_addr);
        Self {
            jingle: jingle.clone(),
            memory: MemoryState::fresh(jingle),
            pc: pc.symbolize(jingle.z3),
        }
    }

    fn apply_control_flow(
        &self,
        op: &PcodeOperation,
        location: ConcretePcodeAddress,
    ) -> Result<SymbolicPcodeAddress<'ctx>, JingleError> {
        self.pc.apply_op(&self.memory, op, location, self.jingle.z3)
    }

    pub fn apply(
        &self,
        op: &PcodeOperation,
        location: ConcretePcodeAddress,
    ) -> Result<Self, JingleError> {
        Ok(Self {
            jingle: self.jingle.clone(),
            memory: self.memory.apply(op)?,
            pc: self.apply_control_flow(op, location)?,
        })
    }
}