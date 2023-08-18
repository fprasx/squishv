pub mod memory;

// TODO: change all printing to hex
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use anyhow::{anyhow, Context};
use thiserror::Error;

use crate::parse::{
    BranchOp, BranchZeroOp, Instruction, LoadImmOp, Program, RegImmOp, RegRegOp, Register, StoreOp,
    UnaryOp,
};

/// Take a snapshot of the registers every `SNAPSHOT_INTERVAL` instructions.
pub const SNAPSHOT_INTERVAL: usize = 1000;

#[rustfmt::skip]
pub const REGISTERS: [Register; 32] = {
    use Register::*;
    [
        x0, ra, sp, gp, tp,
        t0, t1, t2, t3, t4, t5, t6,
        a0, a1, a2, a3, a4, a5, a6, a7,
        s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11,
    ]
};

/// A snapshot of the registers at one point in time.
#[rustfmt::skip]
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct RegisterSnapshot {
    pc: i32,
    
    x0: i32, ra: i32, sp: i32, gp: i32, tp: i32,
    
    t0: i32, t1: i32, t2: i32, t3: i32,
    t4: i32, t5: i32, t6: i32,
    
    a0: i32, a1: i32, a2: i32, a3: i32,
    a4: i32, a5: i32, a6: i32, a7: i32,
    
    s0: i32, s1: i32, s2: i32, s3: i32, s4: i32,
    s5: i32, s6: i32, s7: i32, s8: i32, s9: i32,
    s10: i32, s11: i32,
}

impl RegisterSnapshot {
    pub fn pc(&self) -> i32 {
        self.pc
    }
}

/// Implements `Index{Mut}<Register>` for RegisterFile and RegisterSnapshot
macro_rules! register_index_impl {
    ($( $reg:ident )*) => {
        impl Index<Register> for RegisterSnapshot {
            type Output = i32;
            fn index(&self, index: Register) -> &Self::Output {
                match index {
                    $(
                        Register::$reg
                            => &self.$reg,
                    )*
                }
            }
        }
        impl IndexMut<Register> for RegisterSnapshot {
            fn index_mut(&mut self, index: Register) -> &mut Self::Output {
                match index {
                    $(
                        Register::$reg
                            => &mut self.$reg,
                    )*
                }
            }
        }
        impl Index<&Register> for RegisterSnapshot {
            type Output = i32;
            fn index(&self, index: &Register) -> &Self::Output {
                match index {
                    $(
                        Register::$reg
                            => &self.$reg,
                    )*
                }
            }
        }
        impl IndexMut<&Register> for RegisterSnapshot {
            fn index_mut(&mut self, index: &Register) -> &mut Self::Output {
                match index {
                    $(
                        Register::$reg
                            => &mut self.$reg,
                    )*
                }
            }
        }
    }
}

register_index_impl! {
    x0 ra sp gp tp
    t0 t1 t2 t3 t4 t5 t6
    a0 a1 a2 a3 a4 a5 a6 a7
    s0 s1 s2 s3 s4 s5 s6 s7 s8 s9 s10 s11
}

#[derive(Debug, Clone, Copy)]
pub enum OverflowBehaviour {
    Wrap,
    Saturate,
    // TODO: might not be the correct word
    Trap,
}

#[derive(Debug, Clone, Copy)]
struct Config {
    /// What to do when overflow happens.
    overflow_mode: OverflowBehaviour,
}

// TODO: use Rc/Arc to make cloning cheaper?
#[derive(Debug, Clone)]
pub struct Executor {
    config: Config,

    /// The PC of the next instruction to execute
    pc: i32,

    /// The number of instructions executed - used to uniquely identify points
    /// in program execution.
    executed: usize,
    program: Program,
    pub regfile: RegisterSnapshot,

    /// Used to store snapshots of the programs state at a certain point in time
    /// for ttd (time-travel debugging)
    snapshots: HashMap<usize, RegisterSnapshot>,
    memory: memory::Memory,
}

/// An update that should be applied to the Executor after executing an instruction
#[derive(Debug, Clone)]
pub struct Update {
    pub nextpc: i32,
    pub diff: Option<Diff>,
}

impl Update {
    /// Don't change a register, just jump to the given `pc`
    fn jump(nextpc: i32) -> Self {
        Update { nextpc, diff: None }
    }
}

/// A diff to apply to the registers or pc
#[derive(Debug, Clone, Copy)]
pub enum Diff {
    Memory { addr: i32, val: i32, op: StoreOp },
    Register { reg: Register, val: i32 },
}

pub type ExecResult<T> = Result<T, ExecError>;

#[derive(Error, Debug)]
pub enum ExecError {
    #[error("attempt to write {value} to x0 (hardwired zero)")]
    WriteToX0 { value: i32 },

    /// An error due to the memory system
    #[error(transparent)]
    Other(#[from] anyhow::Error),

    /// Returned when we've hit a breakpoint. It is safe to continue after this.
    #[error("breakpoint hit")]
    BreakPoint,

    #[error("execution finished")]
    Finished,
}

impl Executor {
    pub fn new(program: Program) -> Self {
        Self {
            config: Config {
                overflow_mode: OverflowBehaviour::Trap,
            },
            pc: 0,
            executed: 0,
            program,
            regfile: Default::default(),
            snapshots: Default::default(),
            memory: Default::default(),
        }
    }

    pub fn set(&mut self, reg: Register, val: i32) -> ExecResult<()> {
        if reg == Register::x0 {
            Err(ExecError::WriteToX0 { value: val })
        } else {
            self.regfile[reg] = val;
            Ok(())
        }
    }

    /// Adds two numbers while respecting the configuration for overflow behaviour.
    fn add(&self, fst: i32, snd: i32) -> anyhow::Result<i32> {
        match self.config.overflow_mode {
            OverflowBehaviour::Wrap => Ok(fst.wrapping_add(snd)),
            OverflowBehaviour::Saturate => Ok(fst.saturating_add(snd)),
            OverflowBehaviour::Trap => fst
                .checked_add(snd)
                .ok_or_else(|| anyhow!("overflow error")),
        }
    }

    /// Left shifts while respecting the configuration for overflow behaviour.
    fn shift_left(&self, fst: i32, shamt: u32) -> anyhow::Result<i32> {
        let (res, overflowed) = fst.overflowing_shl(shamt);
        match self.config.overflow_mode {
            OverflowBehaviour::Trap if overflowed => Err(anyhow!("overflow error")),
            _ => Ok(res),
        }
    }

    /// Logical right shifts while respecting the configuration for overflow behaviour.
    fn shift_right_logical(&self, fst: i32, shamt: u32) -> anyhow::Result<i32> {
        // right shifts are arithmetic on signed integers and logical on unsigned integers
        let (res, overflowed) = (fst as u32).overflowing_shr(shamt);
        match self.config.overflow_mode {
            OverflowBehaviour::Trap if overflowed => Err(anyhow!("overflow error")),
            _ => Ok(res as i32),
        }
    }

    /// Arithmetic right shifts while respecting the configuration for overflow behaviour.
    fn shift_right_arithmetic(&self, fst: i32, shamt: u32) -> anyhow::Result<i32> {
        // right shifts are arithmetic on signed integers and logical on unsigned integers
        let (res, overflowed) = fst.overflowing_shr(shamt);
        match self.config.overflow_mode {
            OverflowBehaviour::Trap if overflowed => Err(anyhow!("overflow error")),
            _ => Ok(res),
        }
    }

    pub fn execute(&mut self) -> ExecResult<Update> {
        let update = if self.program.at(self.pc).is_none() {
            Err(ExecError::Finished)?
        } else {
            self._execute()?
        };

        if let Some(diff) = update.diff {
            match diff {
                Diff::Memory { addr, val, op } => {
                    self.memory.store(addr, val, op)?;
                }
                Diff::Register { reg, val } => {
                    if reg != Register::x0 {
                        self.regfile[reg] = val;
                    } else {
                        Err(ExecError::WriteToX0 { value: val })?
                    }
                }
            }
        };

        // only advance pc after applying the diff succeeds (we want to stay stuck
        // if execution fails)
        self.pc = update.nextpc;

        Ok(update)
    }

    // Should only be called if the program is not done executing. This is
    // so that we can use the ? in the Result monad. Self::execute is mostly a
    // wrapper around this function that first checks if we are done executing,
    // and only calls _execute and applies the result if not. This, way we
    // dont have to return Option<anyhow::Result<Update>>, which make make the ?
    // apply to options.
    fn _execute(&mut self) -> anyhow::Result<Update> {
        // See contract for calling funtion
        let asm = self
            .program
            .at(self.pc)
            .expect("program is not done executing");

        // If necessary, take a snapshot _before_ proceeding
        if self.executed % SNAPSHOT_INTERVAL == 0 {
            if let Some(prev) = self.snapshots.insert(self.executed, self.regfile.clone()) {
                // Sanity check that that we're still computing the same result
                // this time around
                assert_eq!(prev, self.regfile);
            }
        }
        // TODO: move the state changes after execution in case there was an error
        self.executed += 1;

        let regs = &self.regfile;
        let pc = self.pc;

        // Helper functions for producing the update
        // Advance the pc by 4 and change the appropriate register
        let next_with = |reg, val| Update {
            nextpc: pc + 4,
            diff: Some(Diff::Register { reg, val }),
        };

        // Advance the pc by 4 and change the appropriate memory location
        let next_mem = |addr, val, op| Update {
            nextpc: pc + 4,
            diff: Some(Diff::Memory { addr, val, op }),
        };

        // Just advance the pc
        let next = Update {
            nextpc: pc + 4,
            diff: None,
        };

        // Some notes:
        // * we mask shift amounts
        let update = match asm {
            // TODO: make sure to test arithmetic/logical shifting
            Instruction::RegImm { rd, r1, imm, op } => {
                let imm = *imm;
                let r1val = regs[r1];
                let val = match op {
                    RegImmOp::Addi => self
                        .add(r1val, imm)
                        .with_context(|| format!("failed to add: {r1} = {r1val}, imm = {imm}"))?,
                    RegImmOp::Sltiu => ((r1val as u32) < (imm as u32)) as i32,
                    RegImmOp::Slli => self.shift_left(r1val, imm as u32).with_context(|| {
                        format!("failed to left shift: {r1} = {r1val}, imm = {imm}",)
                    })?,
                    RegImmOp::Srli => {
                        self.shift_right_logical(r1val, imm as u32)
                            .with_context(|| {
                                format!(
                                    "failed to logical right shift: {r1} = {r1val}, imm = {imm}",
                                )
                            })?
                    }
                    RegImmOp::Srai => {
                        self.shift_right_arithmetic(r1val, imm as u32)
                            .with_context(|| {
                                format!(
                                    "failed to arithmetic right shift: {r1} = {r1val}, imm = {imm}",
                                )
                            })?
                    }
                    RegImmOp::Slti => (regs[r1] < imm) as i32,
                    RegImmOp::Xori => regs[r1] ^ imm,
                    RegImmOp::Ori => regs[r1] | imm,
                    RegImmOp::Andi => regs[r1] & imm,
                };
                next_with(*rd, val)
            }
            Instruction::RegReg { rd, r1, r2, op } => {
                let r1val = regs[r1];
                let r2val = regs[r2];
                let val = match op {
                    RegRegOp::Add => self.add(r1val, r2val).with_context(|| {
                        format!("failed to add: {r1} = {r1val}, {r2} = {r2val}")
                    })?,
                    RegRegOp::Sub => self.add(r1val, -r2val).with_context(|| {
                        format!("failed to subtract: {r1} = {r1val}, {r2} = {r2val}")
                    })?,
                    RegRegOp::Sll => self.shift_left(r1val, r2val as u32).with_context(|| {
                        format!("failed to left shift: {r1} = {r1val}, {r2} = {r2val}")
                    })?,
                    RegRegOp::Srl => {
                        self.shift_right_logical(r1val, r2val as u32)
                            .with_context(|| {
                                format!(
                                    "failed to logical right shift: {r1} = {r1val}, {r2} = {r2val}",
                                )
                            })?
                    }

                    RegRegOp::Sra => self
                        .shift_right_arithmetic(r1val, r2val as u32)
                        .with_context(|| {
                            format!(
                                "failed to arithmetic right shift: {r1} = {r1val}, {r2} = {r2val}",
                            )
                        })?,
                    RegRegOp::Sltu => ((r1val as u32) + (r2val as u32)) as i32,
                    RegRegOp::Slt => (r1val < r2val) as i32,
                    RegRegOp::Xor => r1val ^ r2val,
                    RegRegOp::Or => r1val | r2val,
                    RegRegOp::And => r1val & r2val,
                };
                next_with(*rd, val)
            }
            Instruction::Load { rd, offset, r1, op } => {
                let addr = self.add(*offset, regs[r1]).with_context(|| {
                    format!(
                        "failed to calculate address: {r1} = {}, offset = {offset}",
                        regs[r1]
                    )
                })?;
                next_with(
                    *rd,
                    self.memory
                        .load(addr, *op)
                        .context("failed to perform load")?,
                )
            }
            Instruction::Store { r2, offset, r1, op } => {
                let addr = self.add(*offset, regs[r1]).with_context(|| {
                    format!(
                        "failed to calculate address: {r1} = {}, offset = {offset}",
                        regs[r1]
                    )
                })?;
                next_mem(addr, regs[r2], *op)
            }
            Instruction::Branch { r1, r2, label, op } => {
                let jump = match op {
                    BranchOp::Beq => regs[r1] == regs[r2],
                    BranchOp::Bne => regs[r1] != regs[r2],
                    BranchOp::Blt => regs[r1] < regs[r2],
                    BranchOp::Bge => regs[r1] >= regs[r2],
                    BranchOp::Bltu => (regs[r1] as u32) < (regs[r2] as u32),
                    BranchOp::Bgeu => (regs[r1] as u32) >= (regs[r2] as u32),
                    BranchOp::Bgt => regs[r1] > regs[r2],
                    BranchOp::Ble => regs[r1] <= regs[r2],
                    BranchOp::Bgtu => (regs[r1] as u32) > (regs[r2] as u32),
                    BranchOp::Bleu => (regs[r1] as u32) <= (regs[r2] as u32),
                };
                if jump {
                    Update::jump(self.program.label(label).unwrap().1)
                } else {
                    next
                }
            }
            Instruction::BranchZero { r1, label, op } => {
                let jump = match op {
                    BranchZeroOp::Beqz => regs[r1] == 0,
                    BranchZeroOp::Bnez => regs[r1] != 0,
                    BranchZeroOp::Bltz => regs[r1] < 0,
                    BranchZeroOp::Bgez => regs[r1] >= 0,
                    BranchZeroOp::Bgtz => regs[r1] > 0,
                    BranchZeroOp::Blez => regs[r1] <= 0,
                };
                if jump {
                    Update::jump(self.program.label(label).unwrap().1)
                } else {
                    next
                }
            }
            Instruction::LoadImm { rd, imm, op } => {
                let val = match op {
                    LoadImmOp::Lui => imm << 12,
                    LoadImmOp::Li => *imm,
                };
                next_with(*rd, val)
            }
            Instruction::Unary { rd, r1, op } => {
                let r1val = regs[r1];
                let val = match op {
                    UnaryOp::Mv => r1val,
                    UnaryOp::Not => !r1val,
                    UnaryOp::Neg => -r1val,
                };
                next_with(*rd, val)
            }
            Instruction::call { label } => Update {
                nextpc: self.program.label(label).unwrap().1,
                diff: Some(Diff::Register {
                    reg: Register::ra,
                    val: self.pc + 4,
                }),
            },
            Instruction::jal { rd, label } => Update {
                nextpc: self.program.label(label).unwrap().1,
                diff: Some(Diff::Register {
                    reg: *rd,
                    val: self.pc + 4,
                }),
            },

            Instruction::jalr { rd, offset, r1 } => Update {
                nextpc: self.add(regs[r1], *offset).with_context(|| "")? & !1,
                diff: Some(Diff::Register {
                    reg: *rd,
                    val: self.pc + 4,
                }),
            },
            Instruction::j { label } => Update::jump(self.program.label(label).unwrap().1),
            Instruction::jr { rs } => Update::jump((regs[rs] & !1) * 2),
            Instruction::ret {} => Update::jump(regs[Register::ra]),
        };
        Ok(update)
    }

    fn revert(&mut self) -> Option<Update> {
        todo!()
    }
}