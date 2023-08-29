pub mod memory;

// TODO: change all printing to hex
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    str::FromStr,
};

use anyhow::{anyhow, Context};

use crate::{
    map,
    parse::{
        BranchOp, BranchZeroOp, Instruction, LoadImmOp, Program, RegImmOp, RegRegOp, Register,
        StoreOp, UnaryOp,
    },
};

use self::memory::MemoryError;

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

/// Configuration levels for a setting indicating whether we should allow it, warn,
/// or deny it.
#[derive(Debug, Clone)]
pub enum ConfigLevel {
    Allow,
    Warn,
    Deny,
}

impl ConfigLevel {
    pub fn is_allowed(&self) -> bool {
        matches!(self, ConfigLevel::Allow)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self, ConfigLevel::Warn)
    }

    pub fn is_forbidden(&self) -> bool {
        matches!(self, ConfigLevel::Deny)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    /// What to do when overflow happens.
    overflow_mode: OverflowBehaviour,
    write_to_x0: ConfigLevel,
}

/// Specialized snapshot of the executor for saving some data before entering a
/// function call.
#[derive(Debug, Clone)]
pub struct FnCallEnter {
    snapshot: RegisterSnapshot,

    /// [`FnCallEnter`]'s need to store the `executed` at which they were taken so
    /// we can reset to a previous state properly. `executed` represents the
    /// number of instructions executed _before_ the executing the instruction
    /// that performs the call.
    executed: usize,

    /// The [`Register`] in which the return address of the function we are calling
    /// is stored.
    ra_register: Register,
}

// TODO: use Rc/Arc to make cloning cheaper?
#[derive(Debug, Clone)]
pub struct Executor {
    pub config: Config,

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
    pub memory: memory::Memory,

    /// This is not quite a stack. Rather, each [`FnCallEnter`] stores the state
    /// of the processor right _before_ the call was made. This way, when the
    /// call finishes, we can compare the before and after states.
    stack: Vec<FnCallEnter>,
}

impl FromStr for Executor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse().context("failed to parse program")?))
    }
}

#[derive(Debug, Clone)]
enum StackOp {
    /// Push a new stack frame. `Register` is the [`Register`] in which the return
    /// address is stored.
    PushStack(Register),

    /// Pop a stack frame. `Register` is the [`Register`] in which the return
    /// address is stored.
    PopStack(Register),
}

impl StackOp {
    /// Generate the reverse stack operation.
    ///
    /// A push becomes a pop and a pop becomes a push. The same register is used.
    pub fn reverse(&self) -> StackOp {
        match self {
            StackOp::PushStack(reg) => StackOp::PopStack(*reg),
            StackOp::PopStack(reg) => StackOp::PushStack(*reg),
        }
    }
}

/// An update that should be applied to the Executor after executing an instruction
#[derive(Debug, Clone)]
pub struct ProcessorUpdate {
    pub nextpc: i32,
    pub diff: Option<Diff>,
}

#[derive(Debug)]
pub struct ExecUpdate {
    pc: i32,
    processor_update: ProcessorUpdate,
    stackop: Option<StackOp>,

    /// These generally get
    warnings: Vec<ExecError>,
}

impl ProcessorUpdate {
    /// Don't change a register, just jump to the given `pc`
    fn jump(nextpc: i32) -> Self {
        ProcessorUpdate { nextpc, diff: None }
    }
}

/// A diff to apply to the registers or pc
#[derive(Debug, Clone, Copy)]
pub enum Diff {
    Memory { addr: i32, val: i32, op: StoreOp },
    Register { reg: Register, val: i32 },
}

pub type ExecResult<T> = Result<T, ExecError>;

/// An execution error.
///
/// Generally we produce [`ExecErrorInner`] during execution and turn it into an
/// [`ExecError`] only in the last step of executiong (commiting).
#[derive(Debug)]
pub struct ExecError {
    /// The pc where the error happened
    pc: i32,

    error: ExecErrorInner,
}

#[derive(Debug)]
pub enum ExecErrorInner {
    // #[error("attempt to write {val} to x0 (hardwired zero)")]
    WriteToX0 {
        val: i32,
    },

    /// An error due to the memory system
    // #[error(transparent)]
    // Other(anyhow::Error),
    Memory(MemoryError),

    /// Returned when we've hit a breakpoint. It is safe to continue after this.
    // #[error("breakpoint hit")]
    BreakPoint,

    // #[error("execution finished")]
    Finished,

    // #[error("reverted back to start state")]
    StartReached,

    Overflow(OverflowError),

    // #[error("calling convention violated: {0:?}")]
    CallingConventionViolation(Vec<CallingConventionError>),
}

impl From<MemoryError> for ExecErrorInner {
    fn from(value: MemoryError) -> Self {
        Self::Memory(value)
    }
}

impl From<OverflowError> for ExecErrorInner {
    fn from(value: OverflowError) -> Self {
        Self::Overflow(value)
    }
}

#[derive(Debug)]
pub enum OverflowError {
    Add { base: i32, adding: i32 },
    Sub { base: i32, adding: i32 },
    ShiftLeft { base: i32, shamt: u32 },
    ShiftRight { base: i32, shamt: u32 },
}

impl From<CallingConventionError> for ExecErrorInner {
    fn from(value: CallingConventionError) -> Self {
        Self::CallingConventionViolation(vec![value])
    }
}

impl From<Vec<CallingConventionError>> for ExecErrorInner {
    fn from(value: Vec<CallingConventionError>) -> Self {
        Self::CallingConventionViolation(value)
    }
}

#[derive(Debug)]
pub enum CallingConventionError {
    /// When a callee saved register is modified and not restored during a call
    // #[error("{reg} was {pre} before pre-call, {post} after returning")]
    ModifiedRegister { reg: Register, pre: i32, post: i32 },

    /// When the return address is saved in one register, but we return to a
    /// return address stored in a different register.
    // #[error("last return address was stored in {save} but returning to address in {other}")]
    ReturnViaOtherReg {
        /// The register our last return address was saved in
        save: Register,

        /// The register being used to get the return address
        other: Register,
    },
}

impl Executor {
    pub fn new(program: Program) -> Self {
        let regfile: RegisterSnapshot = RegisterSnapshot {
            sp: 0x40000000, // Halfway up in the address space
            ..Default::default()
        };
        Self {
            config: Config {
                overflow_mode: OverflowBehaviour::Trap,
                write_to_x0: ConfigLevel::Warn,
            },
            pc: 0,
            executed: 0,
            program,
            regfile: regfile.clone(),
            // Start with one snapshot/frame so that we have a state to reset to
            // before we have even executed an instruction. We have to do this
            // because we take snapshots in self.commit
            snapshots: map!(0 => regfile.clone()),
            stack: vec![FnCallEnter {
                snapshot: regfile,
                executed: 0,
                ra_register: Register::ra,
            }],
            memory: Default::default(),
        }
    }

    /// The instruction the executor is about to execute.
    pub fn current(&self) -> Option<Instruction> {
        self.program.at(self.pc).cloned()
    }

    pub fn stack(&self) -> &Vec<FnCallEnter> {
        &self.stack
    }

    pub fn run(&mut self) -> ExecResult<()> {
        loop {
            match self.execute() {
                Ok(_) => continue,
                Err(ExecError {
                    pc,
                    error: ExecErrorInner::Finished,
                }) => return Ok(()),
                other => return other.map(|_| ()),
            }
        }
    }

    pub fn set(&mut self, reg: Register, val: i32, pc: i32) -> ExecResult<()> {
        if reg == Register::x0 {
            Err(ExecError {
                pc,
                error: ExecErrorInner::WriteToX0 { val },
            })
        } else {
            self.regfile[reg] = val;
            Ok(())
        }
    }

    /// Adds two numbers while respecting the configuration for overflow behaviour.
    fn add(&self, fst: i32, snd: i32) -> Result<i32, ExecErrorInner> {
        match self.config.overflow_mode {
            OverflowBehaviour::Wrap => Ok(fst.wrapping_add(snd)),
            OverflowBehaviour::Saturate => Ok(fst.saturating_add(snd)),
            OverflowBehaviour::Trap => fst.checked_add(snd).ok_or_else(|| {
                ExecErrorInner::Overflow(OverflowError::Add {
                    base: fst,
                    adding: snd,
                })
            }),
        }
    }

    /// Left shifts while respecting the configuration for overflow behaviour.
    fn shift_left(&self, fst: i32, shamt: u32) -> Result<i32, ExecErrorInner> {
        let (res, overflowed) = fst.overflowing_shl(shamt);
        match self.config.overflow_mode {
            OverflowBehaviour::Trap if overflowed => {
                Err(ExecErrorInner::Overflow(OverflowError::ShiftLeft {
                    base: fst,
                    shamt,
                }))
            }
            _ => Ok(res),
        }
    }

    /// Logical right shifts while respecting the configuration for overflow behaviour.
    fn shift_right_logical(&self, fst: i32, shamt: u32) -> Result<i32, ExecErrorInner> {
        // right shifts are arithmetic on signed integers and logical on unsigned integers
        let (res, overflowed) = (fst as u32).overflowing_shr(shamt);
        match self.config.overflow_mode {
            OverflowBehaviour::Trap if overflowed => {
                Err(ExecErrorInner::Overflow(OverflowError::ShiftRight {
                    base: fst,
                    shamt,
                }))
            }
            _ => Ok(res as i32),
        }
    }

    /// Arithmetic right shifts while respecting the configuration for overflow behaviour.
    fn shift_right_arithmetic(&self, fst: i32, shamt: u32) -> Result<i32, ExecErrorInner> {
        // right shifts are arithmetic on signed integers and logical on unsigned integers
        let (res, overflowed) = fst.overflowing_shr(shamt);
        match self.config.overflow_mode {
            OverflowBehaviour::Trap if overflowed => {
                Err(ExecErrorInner::Overflow(OverflowError::ShiftRight {
                    base: fst,
                    shamt,
                }))
            }
            _ => Ok(res),
        }
    }

    /// Takes an [`Update`] and applies it to the [`Executor`].
    ///
    /// If the commit fails (for example, due to a memory error), the executor's
    /// state will not be changed.
    fn commit(&mut self, update: &ExecUpdate) -> ExecResult<()> {
        // If we are returning, all caller-saved registers should be the same
        if let Some(StackOp::PopStack(reg)) = update.stackop {
            let mut violations = vec![];
            let frame = &self.stack.last().unwrap();

            // Make sure we're returning via the same register
            if frame.ra_register != reg {
                violations.push(CallingConventionError::ReturnViaOtherReg {
                    save: frame.ra_register,
                    other: reg,
                });
            }

            let diff = self.regfile.check(&frame.snapshot);
            if let Some(diff) = diff {
                violations.extend(
                    diff.iter()
                        .map(|reg| CallingConventionError::ModifiedRegister {
                            reg: *reg,
                            pre: frame.snapshot[reg],
                            post: self.regfile[reg],
                        }),
                )
            }

            if !violations.is_empty() {
                Err(ExecError {
                    pc: self.pc,
                    error: ExecErrorInner::CallingConventionViolation(violations),
                })?
            }

            self.stack.pop().unwrap();
        }

        // Since we don't want to modify the executor if applying the diff fails,
        // save some state now, and we'll use it after if everything goes well.
        let executed = self.executed;
        let snapshot = self.regfile.clone();

        // Apply the change
        if let Some(diff) = update.processor_update.diff {
            match diff {
                Diff::Memory { addr, val, op } => {
                    if let Err(error) = self.memory.store(addr, val, op) {
                        Err(ExecError {
                            pc: self.pc,
                            error: error.into(),
                        })?
                    }
                }
                Diff::Register { reg, val } => {
                    // Checking for writes to x0 is handled in calculate_update
                    self.regfile[reg] = val;
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

        // Retroactively record the state of the processor from *before* making
        // the call (aka applying the change)
        if let Some(StackOp::PushStack(ra_register)) = update.stackop {
            let mut snapshot = snapshot.clone();
            // We actually do want to save the modified register storing the return
            // address since this is the last thing to happen before the call.
            snapshot[ra_register] = self.regfile[ra_register];
            self.stack.push(FnCallEnter {
                snapshot,
                executed,
                ra_register,
            })
        }

        self.pc = update.processor_update.nextpc;
        self.executed += 1;

        Ok(())
    }

    pub fn execute(&mut self) -> ExecResult<ExecUpdate> {
        let Some(asm) = self.program.at(self.pc) else {
            return Err(ExecError { pc: self.pc, error: ExecErrorInner::Finished });
        };
        let update = self
            .calculate_update(asm)
            .map_err(|error| ExecError { pc: self.pc, error })?;
        self.commit(&update)?;
        Ok(update)
    }

    /// Stateless function that returns an [`ExecUpdate`] to produce the next
    /// [`Executor`] state.
    fn calculate_update(&self, asm: &Instruction) -> Result<ExecUpdate, ExecErrorInner> {
        let regs = &self.regfile;
        let pc = self.pc;

        let mut update = ExecUpdate {
            pc,
            processor_update: ProcessorUpdate {
                nextpc: -1,
                diff: None,
            },
            stackop: None,
            warnings: vec![],
        };

        // Helper functions for setting the correct update
        // Advance the pc by 4 and change the appropriate register
        let next_with = |reg, val| ProcessorUpdate {
            nextpc: pc + 4,
            diff: Some(Diff::Register { reg, val }),
        };

        // Advance the pc by 4 and change the appropriate memory location
        let next_mem = |addr, val, op| ProcessorUpdate {
            nextpc: pc + 4,
            diff: Some(Diff::Memory { addr, val, op }),
        };

        // Just advance the pc
        let next = ProcessorUpdate {
            nextpc: pc + 4,
            diff: None,
        };

        let processor_update = match asm {
            Instruction::RegImm { rd, r1, imm, op } => {
                let imm = *imm;
                let r1val = regs[r1];
                let val = match op {
                    RegImmOp::Addi => self.add(r1val, imm)?,
                    RegImmOp::Sltiu => ((r1val as u32) < (imm as u32)) as i32,
                    RegImmOp::Slli => self.shift_left(r1val, imm as u32)?,
                    RegImmOp::Srli => self.shift_right_logical(r1val, imm as u32)?,
                    RegImmOp::Srai => self.shift_right_arithmetic(r1val, imm as u32)?,
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
                    RegRegOp::Add => self.add(r1val, r2val)?,
                    RegRegOp::Sub => self.add(r1val, -r2val)?,
                    RegRegOp::Sll => self.shift_left(r1val, r2val as u32)?,
                    RegRegOp::Srl => self.shift_right_logical(r1val, r2val as u32)?,
                    RegRegOp::Sra => self.shift_right_arithmetic(r1val, r2val as u32)?,
                    RegRegOp::Sltu => ((r1val as u32) + (r2val as u32)) as i32,
                    RegRegOp::Slt => (r1val < r2val) as i32,
                    RegRegOp::Xor => r1val ^ r2val,
                    RegRegOp::Or => r1val | r2val,
                    RegRegOp::And => r1val & r2val,
                };
                next_with(*rd, val)
            }
            Instruction::Load { rd, offset, r1, op } => {
                let addr = self.add(*offset, regs[r1])?;
                let val = self.memory.load(addr, *op)?;
                next_with(*rd, val)
            }
            Instruction::Store { r2, offset, r1, op } => {
                let addr = self.add(*offset, regs[r1])?;
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
                    ProcessorUpdate::jump(self.program.label(label).unwrap())
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
                    ProcessorUpdate::jump(self.program.label(label).unwrap())
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
            Instruction::call { label } => {
                update.stackop = Some(StackOp::PushStack(Register::ra));
                ProcessorUpdate {
                    nextpc: self.program.label(label).unwrap(),
                    diff: Some(Diff::Register {
                        reg: Register::ra,
                        val: self.pc + 4,
                    }),
                }
            }
            Instruction::jal { rd, label } => {
                update.stackop = Some(StackOp::PushStack(*rd));
                ProcessorUpdate {
                    nextpc: self.program.label(label).unwrap(),
                    diff: Some(Diff::Register {
                        reg: *rd,
                        val: self.pc + 4,
                    }),
                }
            }
            Instruction::jalr { rd, offset, r1 } => {
                update.stackop = Some(StackOp::PushStack(*rd));
                let nextpc = self.add(regs[r1], *offset)?;
                ProcessorUpdate {
                    nextpc,
                    diff: Some(Diff::Register {
                        reg: *rd,
                        val: self.pc + 4,
                    }),
                }
            }
            Instruction::la { rd, label } => next_with(*rd, self.program.label(label).unwrap()),
            Instruction::j { label } => ProcessorUpdate {
                nextpc: self.program.label(label).unwrap(),
                diff: None,
            },
            Instruction::jr { rs } => {
                update.stackop = Some(StackOp::PopStack(*rs));
                ProcessorUpdate {
                    nextpc: regs[rs],
                    diff: None,
                }
            }
            Instruction::ret {} => {
                update.stackop = Some(StackOp::PopStack(Register::ra));
                ProcessorUpdate {
                    nextpc: regs[Register::ra],
                    diff: None,
                }
            }
        };

        // Make sure writing to x0 follows the config
        if let Some(Diff::Register { reg, val }) = processor_update.diff {
            if reg == Register::ra {
                match self.config.write_to_x0 {
                    ConfigLevel::Allow => (),
                    ConfigLevel::Warn => update.warnings.push(ExecError {
                        pc: self.pc,
                        error: ExecErrorInner::WriteToX0 { val },
                    }),
                    ConfigLevel::Deny => Err(ExecErrorInner::WriteToX0 { val })?,
                }
            }
        }

        update.processor_update = processor_update;
        Ok(update)
    }

    /// Revert one instruction, returning false if we are already at the start.
    pub fn revert(&mut self) -> bool {
        // A key observation is that if we reached some state, at all points
        // leading up to that state, we had a valid execution. Therefore, if
        // we _correctly_ revert to a certain execution point, we can unwrap all
        // the execution results leading up to the original point because we know
        // they didn't causes errors.

        if self.executed == 0 {
            return false;
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
        let asm = self.program.at(self.pc).unwrap();
        let forward = self.calculate_update(asm).unwrap();
        let diff = forward.processor_update.diff.map(|diff| match diff {
            Diff::Memory { addr, val, op } => todo!(),
            Diff::Register { reg, .. } => Diff::Register {
                reg,
                val: self.regfile[reg],
            },
        });

        let stackop = forward.stackop.map(|op| StackOp::reverse(&op));

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn mem() {
        let mut exec = indoc! {"
            li a0, 0x100
            li a1, 0xaabbccdd
            sw a1, 0(a0)
        "}
        .parse::<Executor>()
        .unwrap();
        exec.run().unwrap();
        assert_eq!(
            exec.memory.mem,
            map! {
                0x100 => 0xdd,
                0x101 => 0xcc,
                0x102 => 0xbb,
                0x103 => 0xaa,
            }
        );
    }

    #[test]
    fn quicksort() {
        let mut program = indoc! {"
            li a0, 0x100
            li a1, 0x100
            li a2, 0x120
            li a3, 4
            sw a3, 0(a0)
            li a3, 3
            sw a3, 4(a0)
            li a3, 2
            sw a3, 8(a0)
            li a3, 1
            sw a3, 0xc(a0)
            // call quicksort
            j done

            // # a0: int* p
            // # a1: start
            // # a2: end
            // quicksort:
            //     bge a1, a2, end # end if start >= end
            //     addi sp, sp, -28
            //     sw ra, 0(sp)  # save ra
            //     sw a0, 4(sp)  # save p
            //     sw a1, 8(sp)  # save start
            //     sw a2, 12(sp) # save end
            //     sw s0, 16(sp)
            //     sw s1, 20(sp)
            //     sw s2, 24(sp)
            //     
            //     call partition
            //     mv s0, a0 # s0 stores q
            //     addi s1, s0, 1  # q + 1
            //     addi s2, s0, -1 # q - 1
            //     
            //     lw a0, 4(sp)
            //     lw a1, 8(sp)
            //     mv a2, s2
            //     call quicksort
            //     
            //     lw a0, 4(sp)
            //     mv a1, s1
            //     lw a2, 12(sp)
            //     call quicksort
            //         
            //     lw ra, 0(sp)
            //     lw a0, 4(sp)
            //     lw a1, 8(sp)
            //     lw a2, 12(sp)
            //     lw s0, 16(sp)
            //     lw s1, 20(sp)
            //     lw s2, 24(sp)
            //     addi sp, sp, 28
            //     
            //     end:
            //         ret
            //
            //
            // # a0: int* p
            // # a1: start
            // # a2: end
            // partition:
            //     addi sp, sp, -52
            //     sw ra, 0(sp)
            //     slli t4, a2, 2 # t0 = end * 4
            //     add t4, t4, a0 # &ptr[end]
            //     lw t5, 0(t4) # pivot
            //
            //     mv a3, a1        # j
            //     addi a4, a1, -1  # tmp
            //     addi a5, a1, -1  # i    
            //     
            //     # >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
            //     j cmp
            //     loop:
            //     
            //     slli t2, a3, 2 # j * 4
            //     add t2, t2, a0 # j offset
            //     lw t3, 0(t2)   # p[j]
            //     
            //     blt t5, t3, skip
            //     addi a5, a5, 1 # i++
            //     
            //     slli t0, a5, 2 # i * 4
            //     add t0, t0, a0 # i offset
            //     
            //     lw a4, 0(t0) # tmp = p[i]
            //     sw t3, 0(t0) # p[i] = p[j]
            //     sw a4, 0(t2) # p[j] = tmp
            //
            //     skip:
            //     addi a3, a3, 1 # j++
            //     cmp:
            //     blt a3, a2, loop
            //     # >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
            //     
            //     addi a5, a5, 1 # i++
            //     slli t0, a5, 2 # i * 4
            //     add t0, t0, a0 # i offset
            //     lw a4, 0(t0)   # tmp = p[i]
            //     sw t5, 0(t0)   # p[i] = end
            //     sw a4, 0(t4)   # p[end] = temp
            //
            //     mv a0, a5
            //
            //     lw ra, 0(sp)
            //     addi sp, sp, 52
            //  
            //     ret
            //
            done:
        "}
        .parse::<Executor>()
        .unwrap();
        program.memory.config.default_value = Some(69);
        program.run().unwrap();
        println!("{:#?}", program.memory)
    }
}
