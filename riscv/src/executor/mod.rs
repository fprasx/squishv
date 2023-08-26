pub mod memory;

// TODO: change all printing to hex
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    str::FromStr,
};

use anyhow::{anyhow, Context};
use thiserror::Error;

use crate::{
    map,
    parse::{
        BranchOp, BranchZeroOp, Instruction, LoadImmOp, Program, RegImmOp, RegRegOp, Register,
        StoreOp, UnaryOp,
    },
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

    /// Compare two [`RegisterSnapshot`] to see if their caller-saved registers
    /// are equal.
    ///
    /// If not, returns [`Some`] with the registers that are different. Otherwise,
    /// returns [`None`].
    pub fn check(&self, other: &RegisterSnapshot) -> Option<Vec<Register>> {
        use Register::*;
        macro_rules! check {
            ($($reg:expr),+ $(,)?) => {
                {
                    let mut different = vec![];
                    $(
                        if self[$reg] != other[$reg] {
                            different.push($reg);
                        }
                    )+
                    different
                }
            };
        }
        let different = check!(sp, s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11);
        if different.is_empty() {
            None
        } else {
            Some(different)
        }
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

#[derive(Debug, Clone)]
struct Config {
    /// What to do when overflow happens.
    overflow_mode: OverflowBehaviour,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    snapshot: RegisterSnapshot,

    /// [`StackFrame`]'s need to store the `executed` at which they were taken so
    /// we can reset to a previous state properly. `executed` represents the
    /// number of instructions executed _before_ the executing the instruction
    /// that performs the call.
    executed: usize,
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
    /// for ttd (time-travel debugging). The index is the number of instructions
    /// executed _before_ taking the snapshot.
    snapshots: HashMap<usize, RegisterSnapshot>,
    memory: memory::Memory,

    /// This is not quite a stack. Rather, each [`StackFrame`] stores the state
    /// of the processor right _before_ the call was made. This way, when the
    /// call finishes, we can compare the before and after states.
    stack: Vec<StackFrame>,
}

impl FromStr for Executor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse().context("failed to parse program")?))
    }
}

#[derive(Debug, Clone)]
enum StackOp {
    /// Push a new stack frame.
    PushStack,

    /// Pop a stack frame.
    PopStack,
}

/// An update that should be applied to the Executor after executing an instruction
#[derive(Debug, Clone)]
pub struct Update {
    pub nextpc: i32,
    pub diff: Option<Diff>,
    stackop: Option<StackOp>,
}

impl Update {
    /// Don't change a register, just jump to the given `pc`
    fn jump(nextpc: i32) -> Self {
        Update {
            nextpc,
            diff: None,
            stackop: None,
        }
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

    #[error("reverted back to start state")]
    StartReached,

    #[error("calling convention violated: {0:?}")]
    CallingConventionViolation(Vec<Register>),
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
            // Start with one snapshot/frame so that we have a state to reset to
            // before we have even executed an instruction. We have to do this
            // because we take snapshots in self.commit
            snapshots: map!(0 => Default::default()),
            stack: vec![StackFrame {
                snapshot: Default::default(),
                executed: 0,
            }],
            memory: Default::default(),
        }
    }

    pub fn stack(&self) -> &Vec<StackFrame> {
        &self.stack
    }

    pub fn run(&mut self) -> ExecResult<()> {
        loop {
            match self.execute() {
                Ok(_) => continue,
                Err(ExecError::Finished) => return Ok(()),
                other => return other.map(|_| ()),
            }
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

    /// Takes an [`Update`] and applies it to the [`Executor`].
    ///
    /// If the commit fails (for example, due to a memory error), the executor's
    /// state will not be changed.
    fn commit(&mut self, update: &Update) -> ExecResult<()> {
        // If we are returning, all caller-saved registers should be the same
        if let Some(StackOp::PopStack) = update.stackop {
            let diff = self.regfile.check(&self.stack.last().unwrap().snapshot);
            if let Some(diff) = diff {
                Err(ExecError::CallingConventionViolation(diff))?
            } else {
                self.stack.pop().unwrap();
            }
        }

        // Since we don't want to modify the executor if applying the diff fails,
        // save some state now, and we'll use it after.
        let executed = self.executed;
        let snapshot = self.regfile.clone();

        // Apply the change
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

        // Retroactively take the snapshot if need be
        if executed % SNAPSHOT_INTERVAL == 0 {
            if let Some(prev) = self.snapshots.insert(executed, snapshot.clone()) {
                // Sanity check that that we're still computing the same result
                // this time around
                assert_eq!(prev, snapshot);
            }
        }

        // Retroactively record the state of the processor before making the
        // call (what we did when we applied the change)
        if let Some(StackOp::PushStack) = update.stackop {
            self.stack.push(StackFrame {
                snapshot: snapshot.clone(),
                executed,
            })
        }

        self.pc = update.nextpc;
        self.executed += 1;

        Ok(())
    }

    pub fn execute(&mut self) -> ExecResult<Update> {
        let update = self
            .next_state()
            .context("failed to execute next instruction")?;
        self.commit(&update)?;
        Ok(update)
    }

    /// Stateless function that returns the [`Update`] to produce the next
    /// [`Executor`] state.
    fn next_state(&self) -> ExecResult<Update> {
        // See contract for calling funtion
        let Some(asm) = self.program.at(self.pc) else {
            return Err(ExecError::Finished);
        };

        let regs = &self.regfile;
        let pc = self.pc;

        // Helper functions for producing the update
        // Advance the pc by 4 and change the appropriate register
        let next_with = |reg, val| Update {
            nextpc: pc + 4,
            diff: Some(Diff::Register { reg, val }),
            stackop: None,
        };

        // Advance the pc by 4 and change the appropriate memory location
        let next_mem = |addr, val, op| Update {
            nextpc: pc + 4,
            diff: Some(Diff::Memory { addr, val, op }),
            stackop: None,
        };

        // Just advance the pc
        let next = Update {
            nextpc: pc + 4,
            diff: None,
            stackop: None,
        };

        let update = match asm {
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
                    Update::jump(self.program.label(label).unwrap())
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
                    Update::jump(self.program.label(label).unwrap())
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
                nextpc: self.program.label(label).unwrap(),
                diff: Some(Diff::Register {
                    reg: Register::ra,
                    val: self.pc + 4,
                }),
                stackop: Some(StackOp::PushStack),
            },
            Instruction::jal { rd, label } => Update {
                nextpc: self.program.label(label).unwrap(),
                diff: Some(Diff::Register {
                    reg: *rd,
                    val: self.pc + 4,
                }),
                stackop: Some(StackOp::PushStack),
            },
            Instruction::jalr { rd, offset, r1 } => Update {
                nextpc: self.add(regs[r1], *offset).with_context(|| "")? & !1,
                diff: Some(Diff::Register {
                    reg: *rd,
                    val: self.pc + 4,
                }),
                stackop: Some(StackOp::PushStack),
            },
            Instruction::j { label } => Update {
                nextpc: self.program.label(label).unwrap(),
                diff: None,
                stackop: None,
            },
            Instruction::jr { rs } => Update {
                nextpc: regs[rs],
                diff: None,
                stackop: Some(StackOp::PopStack),
            },

            Instruction::ret {} => Update {
                nextpc: regs[Register::ra],
                diff: None,
                stackop: Some(StackOp::PopStack),
            },
        };
        Ok(update)
    }

    /// Function that reverts one instruction, returning the [`Update`] that
    /// does the necessary changes.
    pub fn revert(&mut self) -> ExecResult<Update> {
        // A key observation is that if we reached some state, at all points
        // leading up to that state, we had a valid execution. Therefore, if
        // we _correctly_ revert to a certain execution point, we can unwrap all
        // the execution results leading up to the original point because we know
        // they didn't causes errors.

        if self.executed == 0 {
            return Err(ExecError::StartReached);
        }

        let executed = self.executed;
        let target = executed - 1;
        let offset_from_snapshot = executed.rem_euclid(SNAPSHOT_INTERVAL);
        let base = executed - offset_from_snapshot;

        // Every `SNAPSHOT_INTERVAL` we take store a snapshot so it should exist.
        let regs = self.snapshots.get(&base).unwrap_or_else(|| {
            panic!("no register snapshot for {base} even though we reached {executed}")
        });

        // Reset the executor to the base state
        self.regfile = regs.clone();
        self.pc = self.regfile.pc;
        self.executed = base;

        // Reset to the correct stack frame too
        let frames = self.stack.partition_point(|frame| frame.executed <= target);
        self.stack.truncate(frames);

        // Execute to the state we want to revert to. We know executed cannot be
        // 0 because we check first thing
        while self.executed < target {
            self.execute().unwrap();
        }

        // Generate the next update so we can reverse it to figure out the diff
        // that reverts from the start state
        let forward = self.next_state().unwrap();
        let diff = forward.diff.map(|diff| match diff {
            Diff::Memory { addr, val, op } => todo!(),
            Diff::Register { reg, .. } => Diff::Register {
                reg,
                val: self.regfile[reg],
            },
        });

        let stackop = forward.stackop.map(|op| match op {
            StackOp::PushStack => StackOp::PopStack,
            StackOp::PopStack => StackOp::PushStack,
        });

        Ok(Update {
            nextpc: self.pc,
            diff,
            stackop,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {}
}
