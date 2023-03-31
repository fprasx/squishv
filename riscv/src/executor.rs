use std::{
    default::Default,
    ops::{Index, IndexMut},
};

use crate::parse::{Instruction, Register};

#[derive(Default)]
struct Executor {
    regfile: RegisterFile,
    updates: Vec<Update>,
}

struct Update {}

impl Executor {
    fn new() -> Self {
        Default::default()
    }

    fn execute(instr: Instruction) -> Update {
        match instr {
            Instruction::addi { rd, r1, imm } => todo!(),
            Instruction::slti { rd, r1, imm } => todo!(),
            Instruction::sltiu { rd, r1, imm } => todo!(),
            Instruction::xori { rd, r1, imm } => todo!(),
            Instruction::ori { rd, r1, imm } => todo!(),
            Instruction::andi { rd, r1, imm } => todo!(),
            Instruction::slli { rd, r1, imm } => todo!(),
            Instruction::srli { rd, r1, imm } => todo!(),
            Instruction::srai { rd, r1, imm } => todo!(),
            Instruction::lui { rd, imm } => todo!(),
            Instruction::li { rd, imm } => todo!(),
            Instruction::add { rd, r1, r2 } => todo!(),
            Instruction::sub { rd, r1, r2 } => todo!(),
            Instruction::sll { rd, r1, r2 } => todo!(),
            Instruction::slt { rd, r1, r2 } => todo!(),
            Instruction::sltu { rd, r1, r2 } => todo!(),
            Instruction::xor { rd, r1, r2 } => todo!(),
            Instruction::srl { rd, r1, r2 } => todo!(),
            Instruction::sra { rd, r1, r2 } => todo!(),
            Instruction::or { rd, r1, r2 } => todo!(),
            Instruction::and { rd, r1, r2 } => todo!(),
            Instruction::lw { rd, mem } => todo!(),
            Instruction::sw { r1, mem } => todo!(),
            Instruction::beq { r1, r2, label } => todo!(),
            Instruction::bne { r1, r2, label } => todo!(),
            Instruction::blt { r1, r2, label } => todo!(),
            Instruction::bge { r1, r2, label } => todo!(),
            Instruction::bltu { r1, r2, label } => todo!(),
            Instruction::bgeu { r1, r2, label } => todo!(),
            Instruction::bgt { r1, r2, label } => todo!(),
            Instruction::ble { r1, r2, label } => todo!(),
            Instruction::bgtu { r1, r2, label } => todo!(),
            Instruction::bleu { r1, r2, label } => todo!(),
            Instruction::beqz { r1, label } => todo!(),
            Instruction::bnez { r1, label } => todo!(),
            Instruction::bltz { r1, label } => todo!(),
            Instruction::bgez { r1, label } => todo!(),
            Instruction::bgtz { r1, label } => todo!(),
            Instruction::blez { r1, label } => todo!(),
            Instruction::mv { rd, r1 } => todo!(),
            Instruction::not { rd, r1 } => todo!(),
            Instruction::neg { rd, r1 } => todo!(),
            Instruction::jal { rd, label } => todo!(),
            Instruction::jalr { rd, addr } => todo!(),
            Instruction::call { label } => todo!(),
            Instruction::pseudo_jal { label } => todo!(),
            Instruction::j { label } => todo!(),
            Instruction::jr { rs } => todo!(),
            Instruction::pseudo_jalr { rs } => todo!(),
            Instruction::ret {} => todo!(),
        }
    }
}

#[rustfmt::skip]
#[derive(Default)]
struct RegisterFile {
   x0: i32, ra: i32, sp: i32, gp: i32, tp: i32,

   t0: i32, t1: i32, t2: i32, t3: i32,
   t4: i32, t5: i32, t6: i32,

   a0: i32, a1: i32, a2: i32, a3: i32,
   a4: i32, a5: i32, a6: i32, a7: i32,

   s0: i32, s1: i32, s2: i32, s3: i32, s4: i32,
   s5: i32, s6: i32, s7: i32, s8: i32, s9: i32,
   s10: i32, s11: i32,
}

macro_rules! register_file_index_impl {
    ($( $reg:ident )*) => {
        impl Index<Register> for RegisterFile {
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

        impl IndexMut<Register> for RegisterFile {
            fn index_mut(&mut self, index: Register) -> &mut Self::Output {
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

register_file_index_impl! {
    x0 ra sp gp tp
    t0 t1 t2 t3 t4 t5 t6
    a0 a1 a2 a3 a4 a5 a6 a7
    s0 s1 s2 s3 s4 s5 s6 s7 s8 s9 s10 s11
}
