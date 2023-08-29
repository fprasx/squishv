use std::io;

use anyhow::Context;
use riscv::{executor::Executor, lex::Lexer, parse::Program};

fn main() -> anyhow::Result<()> {
    let mut program = indoc::indoc! {"
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
        call partition
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



        # a0: int* p
        # a1: start
        # a2: end
        partition:
            addi sp, sp, -52
            sw ra, 0(sp)
            slli t4, a2, 2 # t0 = end * 4
            add t4, t4, a0 # &ptr[end]
            lw t5, 0(t4) # pivot

            mv a3, a1        # j
            addi a4, a1, -1  # tmp
            addi a5, a1, -1  # i    
            
            # >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
            j cmp
            loop:
            
            slli t2, a3, 2 # j * 4
            add t2, t2, a0 # j offset
            lw t3, 0(t2)   # p[j]
            
            blt t5, t3, skip
            addi a5, a5, 1 # i++
            
            slli t0, a5, 2 # i * 4
            add t0, t0, a0 # i offset
            
            lw a4, 0(t0) # tmp = p[i]
            sw t3, 0(t0) # p[i] = p[j]
            sw a4, 0(t2) # p[j] = tmp

            skip:
            addi a3, a3, 1 # j++
            cmp:
            blt a3, a2, loop
            # >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
            
            addi a5, a5, 1 # i++
            slli t0, a5, 2 # i * 4
            add t0, t0, a0 # i offset
            lw a4, 0(t0)   # tmp = p[i]
            sw t5, 0(t0)   # p[i] = end
            sw a4, 0(t4)   # p[end] = temp

            mv a0, a5

            lw ra, 0(sp)
            addi sp, sp, 52
         
            ret

        done:
    "}
    .parse::<Executor>()
    .unwrap();
    program.memory.config.default_value = Some(69);
    repl(program);
    Ok(())
}

fn repl(mut exec: Executor) {
    use crossterm::{
        execute,
        terminal::{Clear, ClearType},
    };
    use std::io::stdout;
    loop {
        let stdin = io::stdin();
        let mut buf = String::new();
        stdin.read_line(&mut buf).unwrap();
        execute!(stdout(), Clear(ClearType::All)).unwrap();
        println!("{}", exec.current().unwrap());
        println!("{:#?}", exec.execute().unwrap());
    }
}
