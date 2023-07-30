'''
generate random assembly instructions for "fuzz" testing
'''

import random
import sys

alphabet = "abcdefghijklmnopqrstuvwxyz_"


def random_label():
    return ''.join(random.choice(alphabet)
                   for _ in range(random.randrange(5, 10)))


def random_num():
    '''
    Returns a stringified hexadecimal or decimal i32
    '''
    num = random.randrange(- (2 ** 31), 2 ** 31 - 1)
    if random.random() < 0.5:
        return hex(num)
    else:
        return str(num)


if __name__ == "__main__":
    iters = 1
    if len(sys.argv) > 1:
        iters = int(sys.argv[1])

    with open("registers.txt", "r") as f:
        regs = list(map(lambda line: line.strip(), f.readlines()))

    items = []
    for _ in range(iters):
        # produce one of every instruction
        for i in ["addi", "slti", "sltiu", "xori",
                  "ori", "andi", "slli", "srli", "srai"]:
            rd, r1 = random.choices(regs, k=2)
            num = random_num()
            items.append(f"{i} {rd}, {r1}, {num}")

        for i in ["add", "sub", "sll", "slt",
                  "sltu", "xor", "srl", "sra", "or", "and"]:
            rd, r1, r2 = random.choices(regs, k=3)
            items.append(f"{i} {rd}, {r1}, {r2}")

        for i in ["beq", "bne", "blt", "bge", "bltu",
                  "bgeu", "bgt", "ble", "bgtu", "bleu"]:
            r1, r2 = random.choices(regs, k=2)
            label = random_label()
            items.append(f"{label}:")
            items.append(f"{i} {r1}, {r2}, {label}")

        for i in ["beqz", "bnez", "bltz", "bgez", "bgtz", "blez"]:
            r1 = random.choice(regs)
            label = random_label()
            items.append(f"{label}:")
            items.append(f"{i} {r1}, {label}")

        for i in ["mv", "neg", "not"]:
            r1, r2 = random.choices(regs, k=2)
            items.append(f"{i} {r1}, {r2}")

        for i in ["lw", "lh", "lb", "sw", "sh", "sb"]:
            r1, r2 = random.choices(regs, k=2)
            num = random_num()
            items.append(f"{i} {r1}, {num}({r2})")

        for i in ["lui", "li"]:
            r1 = random.choice(regs)
            num = random_num()
            items.append(f"{i} {r1}, {num}")

        # call        { label: &'a str },
        label = random_label()
        items.append(f"{label}:")
        items.append(f"call {label}")

        # jal         { rd: register, label: &'a str },
        rd = random.choice(regs)
        label = random_label()
        items.append(f"{label}:")
        items.append(f"jal {rd}, {label}")
        label = random_label()
        # register not provided variant, ra assumed
        label = random_label()
        items.append(f"{label}:")
        items.append(f"jal {label}")

        # jalr        { rd: register, offset: i32, r1: register },
        rd, r1 = random.choices(regs, k=2)
        num = random_num()
        items.append(f"jalr {rd}, {num}({r1})")
        # register not provided variant, ra assumed
        rd = random.choice(regs)
        items.append(f"jalr {rd}")

        # j           { label: &'a str },
        label = random_label()
        items.append(f"{label}:")
        items.append(f"j {label}")

        # jr          { rs: register },
        rs = random.choice(regs)
        items.append(f"jr {rs}")

        # ret         {},
        items.append("ret")

    random.shuffle(items)
    for i in items:
        print(i)
