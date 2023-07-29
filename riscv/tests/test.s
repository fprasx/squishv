# RETURNS: Nothing
  slli  a2, a0, 1
            lw a6, 4(a4) # p[i+1]
            j for # go to top of for loop
    ret
    lw ra, 0(sp)
    call partition
    lw s1, 8(sp)

    skip:
    li s1, 0 # x

    li a2, 0x60004004   # GPIO_OUT_ADDR
    lw a0, 4(sp)
    mv s7, a6 # down
    bne a2, a1, looping # continue looping if i < 8
                    sw a6, 24(sp)
config_input1:
    call getPixel # getPixel(game_board, x+1, y-1)

    li a5, 1        # just store 1
    add t2, t2, a0 # &ptr[i]
    # modding y's
    mv a4, a1        # j
    lw ra, 0(sp)
            addi a3, a3, 1 # i++

    sw a7, 0(a4)    # load it back

    # RISC-V calling convention.
    
    sw a4, 24(sp)
    sll a5, a4, a0 # a5 = 1 << pin_num
    li a2, 0            # 'i' for for loop
    sw a2, 12(sp)
            mv a1, s1
    sw a4, 0(t4)   # p[end] = temp
    lw s0, 4(sp)
                sw a7, 4(a4) # p[i+1] = p[i]

    andi a0, a1, 0x1
    li a2, 0x60009004   # IO_MUX_GPIOn_ADDR
                    lw a1, 4(sp)

    
  add   a0, a2, a0
    
    lw t1, 32(sp)
    sw a1, 8(sp)
    ret
        addi s1, s1, 1 # x++
    addi sp, sp, -4
        inner:
    lw a5, 0(t2) # tmp = ptr[i]

    lw t2, 36(sp)
    li a4, 1
    add a6, a6, a5  # a6 = a6 + a5
    li a6, 1
    // >>>>>>>>>>>>>>>>>>>>>>>>>
    # store a registers in s registers
    lw a5, 0(t0)
    sw a2, 12(sp)
                    lw a2, 8(sp)
    lw a3, 16(sp)
    lw a1, 8(sp)
# updateBoard
        inner_cmp:
# writing 0 to a ping
    loop1:
    sw a0, 4(sp)  # save p
    sw zero, 0(a4)      # write 0 to memory address
                    jal arrayViz
    sw s2, 12(sp)
    add a4, a4, a0 # add it to the pointer to the array
    call arrayViz


    sw t4, 44(sp)
    add s3, s3, a0

    lw a6, 0(a3) # load value in GPIO_ENABLE_ADDR
# RETURNS: Nothing
    srli a6, a6, 29
    
    li a4, 2
    sw a4, 0(t2) # p[j] = tmp
    

                # doing the swap
    lw t3, 40(sp)
    lw a7, 0(a4)    # load the value

    and a4, a4, a3 
# a1: temporary output buffer (for new board)
    sw a4, 0(a2)
                mv a2, s2
    lw a4, 0(a2) # a4 holds the GPIO out value
    addi sp, sp, 28
    # Now we write 1 to the pin
    mv a2, s2
    addi a5, a2, -1 # up
    add t0, t0, a0 # i offset
    beqz a3, set_pixel_zero
# a3: left neighbor index, a4: right neighbor index, a5: up neighbor index,
    ret
    j cmp
    srli a3, a3, 27

        
    slli t0, a5, 2 # i * 4
    sll a6, a6, a1 # a6: 1 << x

while:
    lw a0, 4(sp)
    sw t5, 0(t0)   # p[i] = end
    lw t3, 40(sp)

    slli t2, a6, 2 # i * 4

    lw t4, 44(sp)
looping: 
# ARGUMENTS a0: screen buffer starting address, a1: x, a2: y, a3: val
    li a2, 0            # 'i' for for loop
# ARGUMENTS a0: screen_buffer starting address
    sw a5, 0(a4)   # store it back
    sw a4, 0(a2)   # store it back
    slli a3, a3, 27
    sw t2, 36(sp)
        blt s1, s5, inner
pinWrite:
                    lw a0, 0(sp)
            noswap:
    li a2, 0 # swapped = 0
    add s3, s3, a0
    li a3, 1 
    sw a1, 8(sp)
# ARGUMENTS a0: screen buffer (current board), a1: temporary output buffer (for new board)
write_zero:
    lw ra, 0(sp)
    sw zero, 0(a4)      # write 0 to memory address
    lw a5, 0(a4)   # read from it to get the correct
                mv a7, a5 # tmp = p[i]
    sll a3, a3, a0 # a3: 1 << pin_num
    lw s3, 16(sp)
    lw a4, 20(sp)

    mv a0, s4
    # >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
    sll a6, a6, a1 # a6: 1 << x
                j over
    sll a6, a6, a1  # a6: 1 << x

    not a3, a3     # a3 = ~a3
    bnez a2, while # go to top of while loop
    
    sll a4, a0, a4  # get the offset, put it in a4

    mv a0, s0
    

    lw t5, 48(sp)
arrayViz:
    slli a6, a5, 9  # a6 = 1 << 9
    mv a2, s7
    bne a2, a1, looping # continue looping if i < 8
# a2: end
    slli a5, a5, 8  # a5 = 1 << 8

            add a4, a4, a0 # a4 now stores ptr + offset
    mv a0, s0
    mv s1, a1 # a1: x
        bge a3, a1, end_for # end the for loop if i >= n
    lw ra, 0(sp)
    mv a6, a5        # i
    
# a2: end
    or a4, a4, a3  # or the GPIO out value with 1 << pin_num
collatz:

    sw s3, 16(sp)

    # your code here
    lw s6, 28(sp)

# C declaration: void setPixel(uint32_t* screen_buffer_addr, uint8_t x, uint8_t y, uint8_t val)
    addi a2, a2, 1      # increment i 
    li a4, 1
            mv a1, s1
    ret
    lw s1, 20(sp)
  bnez  a1, Else
    mv a2, s2
    ret
    add a4, a4, a2  # add the offset, a4: ptr to write to

    li a6, 1
Then:
    mv s0, a0 # s0 stores q
    // >>>>>>>>>>>>>>>>>>>>>>>>>

    sw a5, 0(t0) # ptr[j] = tmp

    loop:
    mv a2, s2
    lw s6, 28(sp)
            two_neighbors:

    sll a4, a0, a4  # get the offset, put it in a4
    addi a4, a4, 1 # j++
    not a3, a3     # a3 = ~a3
    slli t0, a6, 2 # i * 4
    call quicksort
    add s3, s3, a0
    sll a3, a3, a0 # a3: 1 << pin_num
    sw a4, 24(sp)
    j cmp

    sll a3, a3, a0 # a3: 1 << pin_num
    // >>>>>>>>>>>>>>>>>>>>>>>>>
    mv a0, a6
    
    sw t0, 28(sp)
    lw a5, 0(a4)   # read from it to get the correct
    ret
    lw a2, 12(sp)
                    sw a5, 20(sp)
    li a2, 0x60009004   # IO_MUX_GPIOn_ADDR
    lw ra, 0(sp)
# RETURNS: Nothing
    mv a1, s5
    sw a6, 0(a3)

# a0: int* p
    andi a0, a3, 1  # a0: a3 & 1
    sw a0, 4(sp)
    sw t1, 0(t2) # ptr[i] = ptr[j]
    sw s6, 28(sp)
    beqz a1, config_input # handle mode = 0
    addi s2, s0, -1 # q - 1
    not a6, a6     # not it so we can zero one bit

                

# checkNeighbors
    sll a5, a4, a0 # a5 = 1 << pin_num
    srl a1, a1, a0      # shift down by pin_num
# RETURNS: total occupied cells in the eight surrounding cells of current (x,y)
    
            call checkNeighbors
    lw t0, 28(sp)
    add a4, a4, a2  # add the offset, a4: ptr to write to
    add t0, t0, a0 # &ptr[j]
    sw s7, 32(sp)
# ARGUMENTS a0: screen_buffer starting address
    lw s5, 24(sp)
    mv a0, s0
# RETURNS: total occupied cells in the eight surrounding cells of (x,y) (game board wraps in x and y)
    sw s2, 24(sp)
    ret

pinWrite2:
    lw a4, 0(t0)   # tmp = p[i]
    addi sp, sp, -52
    li a5, 1        # just store 1
    lw a1, 8(sp)
  srai  a0, a0, 1
    ret
    
                    sw a4, 16(sp)
        end_for:

    ret

    sw a3, 16(sp)
    addi sp, sp, -28

    sw a7, 0(a4)    # load it back
        mv s1, zero # x = 0
                    addi sp, sp, -36
    slli t2, a3, 2 # j * 4
    sw s0, 16(sp)
    mv s3, a0 # current_board
    lw a4, 0(t0) # tmp = p[i]
    sw s5, 24(sp)
    mv s4, a1 # new_board
                    sw ra, 32(sp)
    
                mv a0, s4
    li a1, 8            # upper bound on for loop
    sw a5, 0(a4)    # write it back
    li a6, 1
    sw s0, 4(sp)
    and a5, a5, a6 # and it in
    mv a0, s0

    add a4, a0, a3      # get address of array element by adding base address + 4*i 
    
    addi sp, sp, -36
set_pixel_zero1:
    addi a1, a1, -1 # n--
config_input:
    mv a1, s4
                call setPixel
    cmp:
# ARGUMENTS a0: pin_num, a1: mode
                    sw a0, 0(sp)
    
    sw a6, 0(a3)
        for:
    ret
    sw s2, 12(sp)

    lw s0, 4(sp)
    srl a1, a1, a0      # shift down by pin_num
    bge a3, t1, skip # skip pivot end >= ptr[j]
    lw t3, 0(t2)   # p[j]
    or a7, a7, a6   # or with the prepared value
                    sw a1, 4(sp)
    
    lw a3, 0(a2)    # a3: screen_buffer[y]
                mv a1, s1


    sw s1, 20(sp)
    addi a1, a1, -1

# RETURNS: 1 if cell occupied, 0 otherwise
                mv a0, s4
    call getPixel # getPixel(game_board, x-1, y+1)
    sw t5, 48(sp)
# RETURNS: Nothing
    mv a1, s1
    mv a1, s1
    call arrayViz

    not a6, a6     # not it so we can zero one bit
    lw a0, 4(sp)
# RETURNS: Nothing
            over:
set_pixel_zero:

    sw t3, 40(sp)
    add t0, t0, a0 # &ptr[end]
    slli a4, a2, 2 # calculating pointer offset: y << 2 = y * 2 
    beqz a1, write_zero # check if value is 0

    addi a4, a1, -1 # right
                    lw a3, 12(sp)
  
    # TODO: Determine left, right, up, and down indices for neighbors.
    sw a2, 12(sp) # save end
    lw a2, 12(sp)
# a1: start
    lw a0, 4(sp)
    addi a5, a5, 1 # i++
 
    add a6, a6, a5  # a6 = a6 + a5
# RETURNS: Nothing
    mv a3, a1        # j
    
    skip1:
    li a1, 8            # upper bound on for loop

    sw ra, 0(sp)
    mv a1, s4
    call getPixel # getPixel(game_board, x-1, y)
# ARGUMENTS a0: pin_num, a1: mode

# ARGUMENTS a0: screen_buffer, a1: x, a2: y
                    lw ra, 32(sp)
    slli a5, a5, 29
    add s3, s3, a0
                bne s6, s5, over # skip if not 2 neighbors
    srl a3, a3, a1  # a3: screen_buffer[y] >>_l x
    slli a6, a6, 29
            beqz s7, over # skip if cell is dead
    sw a4, 0(a2)
pinSetup:
    sw s5, 24(sp)
    sw s1, 8(sp)
            mv s7, a0 # s7 now holds pixel value
    or a6, a6, a5 # or with 1 << pin_num

setPixel1:
    
    sw ra, 0(sp)
# RETURNS: Nothing
    // >>>>>>>>>>>>>>>>>>>>>>>>>
    mv a2, s6
    sw a3, 0(t0)
    lw t1, 32(sp)
# ARGUMENTS a0: game_board, a1: x index, a2: y index

    mv a0, s0
    lw a3, 0(t0) # pivot
    sw s4, 20(sp)

    sw a5, 24(sp)
    lw s7, 32(sp)
    and a5, a5, a6 # and it in
    

    slli a3, a2, 2      # calculate 4*i
    call quicksort
    add s3, s3, a0
    # First index correct integer


            bge a6, a5, noswap # go to noswap
        j inner_cmp # TODO: might not need
    addi sp, sp, -36
 
# a0: int* p
# ARGUMENTS a0: screen buffer (current board),
    lw s5, 24(sp)
                mv a2, s2
    sw s3, 16(sp)
    lw t2, 36(sp)
    andi a0, a1, 0x1
    addi a5, a4, -1  # tmp
    li s2, 0 # y
 
    mv s6, a5 # up

    addi t0, t0, 0
    mv a0, s0
# updateBoard
    addi a6, a6, 1 # i++
tallyNeighbors:
    call tallyNeighbors # tallyNeighbors(game_board, x, y, left, right, up, down)
  j     End
    blt a3, a2, loop
    li a3, 0 # i = 0
# C declaration: int pinRead(int pin_num)
    
    and a4, a4, a3 
    li a3, 0x60004020   # GPIO_ENABLE_ADDR
    lw s2, 24(sp)
    sw a5, 0(a4)    # write it back
    addi sp, sp, 52
    lw s2, 12(sp)
    
  ret
    or a4, a4, a3  # or the GPIO out value with 1 << pin_num
    lw a2, 12(sp)
    sw t3, 0(t0) # p[i] = p[j]
    addi sp, sp, 36
    
    ret
    
                li s5, 2
    ret
    li a6, 1
    lw s0, 16(sp)
    sw ra, 0(sp)
                li a3, 1


    addi a5, a1, -1  # i    
            call getPixel
    # modding x's
    j outer_cmp
    ret
    mv a0, s0

    slli t0, a5, 2 # i * 4
  
    mv s2, a2 # a2: y
    
# ARGUMENTS a0: screen buffer starting address, a1: x, a2: y, a3: val
    ret
    sw t1, 32(sp)
    sw t3, 40(sp)
    outer_cmp:
        blt s2, s5, outer
            mv a0, s3
                    sw a3, 12(sp)
checkNeighbors:
    lw a1, 8(sp)
    slli t4, a2, 2 # t0 = end * 4
    lw a1, 0(a2)        # a1: *GPIO_IN_ADDR
    lw s3, 16(sp)
    blt a4, a2, loop
    lw a1, 8(sp)
    li a2, 0x60004004   # GPIO_OUT_ADDR
# RETURNS: Nothing
    # instructions that:

    lw a0, 4(sp)
# C declaration: void eraseBuffer(uint32_t* screen_buffer_addr)
    ret
    

    mv a2, s6
    lw t4, 44(sp)
    li a3, 1 
# C declaration: void pinSetup(int pin_num, int mode)
partition:
# ARGUMENTS a0: game_board, a1: current x index, a2: current y index,
    addi sp, sp, 36
    # 1. Increment/decrement the stack pointer

    call getPixel # getPixel(game_board, x, y-1)
    mv a1, s4
                    lw a7, 28(sp)
    mv s0, a0 # s0: game_board
    addi a6, a2, 1 # down
    srli a5, a5, 29
    sw t4, 44(sp)
# RETURNS: Nothing
    # >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
    # TODO: This procedure is functionally correct, but doesn't follow
                # stuff for printing
                li a3, 1
# C declaration: void setPixel(uint32_t* screen_buffer_addr, uint8_t x, uint8_t y, uint8_t val)
    mv s4, a3 # left

    addi sp, sp, 4
    add s3, s3, a0
bubblesort:
    slli a3, a2, 2      # calculate 4*i
    add a4, a0, a3      # get address of array element by adding base address + 4*i 
    addi a3, a3, 1 # j++
    slli a5, a5, 8  # a5 = 1 << 8
    ret
Else:
    sw s4, 20(sp)
                    lw a4, 16(sp)
pinRead:
# a6: down neighbor index
    add a4, a4, a0 # add it to the pointer to the array
    beqz a1, config_input # handle mode = 0
    bge a1, a2, end # end if start >= end

    mv a1, s5
    sw s7, 32(sp)
    li s3, 0  # s3: tally


    call getPixel # getPixel(game_board, x+1, y)
                    lw a5, 20(sp)

    outer:
    slli t0, a2, 2 # t0 = end * 4
    slli t0, a4, 2 # t0 = j * 4
    # Now we write 1 to the pin
    
    lw a2, 12(sp)
End:
    
                    addi sp, sp, 36
            mv s6, a0 # s6 now holds tally
                    lw a6, 24(sp)
    li a3, 0x60004020   # GPIO_ENABLE_ADDR
    or a5, a5, a6   # or a6 into a5 to set the bit
    
    lw s1, 8(sp)
    addi a3, a1, 1 # left
    lw s2, 12(sp)
    lw a6, 0(a3) # load value in GPIO_ENABLE_ADDR
    lw a3, 16(sp)
    
    end:
    call eraseBuffer


    # Make this procedure follow calling convention. You may only add
            slli a4, a3, 2 # offset = i << 2 = i * 4
            bne s6, s5, two_neighbors # skip if not 3 neighbors
    lw a4, 0(a2) # a4 holds the GPIO out value
# C declaration: void eraseBuffer(uint32_t* screen_buffer_addr)
    # Handle mode = 1
    slli a6, a5, 9  # a6 = 1 << 9
    
    call getPixel # getPixel(game_board, x+1, y+1)
                    sw a7, 28(sp)
    sw a1, 8(sp)  # save start
    call getPixel # getPixel(game_board, x-1, y-1)
setPixel:
# writing 0 to a ping
# RETURNS bit read from GPIO pin 
    li a2, 0x6000403C   # GPIO_IN_ADDR
looping1: 
    lw a4, 20(sp)
        li s5, 8

    sw a4, 0(a2)   # store it back
            mv a2, s2
    sw a3, 16(sp)
    sll a6, a6, a1  # a6: 1 << x
    ret
# getPixel
    lw a4, 0(a2) # a4 holds the GPIO out value

    addi a5, a5, 1 # i++

    # First index correct integer
    addi s1, s0, 1  # q + 1
pinSetup2:
    mv a2, s7

    addi a6, a6, 1 # i++
    add t2, t2, a0 # j offset
    # 2. Put elements on the stack

    mv a1, s1


    srli a4, a4, 27
    add t0, t0, a0 # i offset
    lw t0, 28(sp)
updateBoard:
    ret
    mv a0, a5
    call getPixel # getPixel(game_board, x, y+1)
                call setPixel
            lw a5, 0(a4) # p[i]
    mv s5, a4 # right
    sw ra, 0(sp)  # save ra
    beqz a3, set_pixel_zero
    sw t5, 48(sp)
    li a3, 1 
    lw s4, 20(sp)
# tallyNeighbors
    

    
    
    slli t0, a2, 2 # t0 = end * 4
            mv a2, s2

# a1: start
                li a2, 1 # swapped = 1
    slli a4, a2, 2 # calculating pointer offset: y << 2 = y * 2 

    sw a0, 4(sp)
    sll a3, a3, a0 # a3: 1 << pin_num
    
    
    li a2, 0x6000403C   # GPIO_IN_ADDR

    # 3. Take elements off the stack
    mv a2, s6
If:
    # Handle mode = 1
# C declaration: void pinWrite(int pin_num, int value)
    add t4, t4, a0 # &ptr[end]
                mv a1, s1
    beqz a1, write_zero # check if value is 0
    or a5, a5, a6   # or a6 into a5 to set the bit
    slli a4, a4, 27
    sw s1, 8(sp)
    or a6, a6, a5 # or with 1 << pin_num
   
    sw a5, 0(t0)
    add s3, s3, a0
eraseBuffer:
    
    ret                 # return from eraseBuffer.section .text     
    lw t5, 0(t4) # pivot
    addi s2, s2, 1 # y++
    sw ra, 0(sp)
    sw s6, 28(sp)
    lw a5, 24(sp)

# ARGUMENTS a0: pin_num, a1: value
            mv a0, s3
    sw t1, 32(sp)
            li s5, 3
    li a4, 2
    addi a4, a1, -1  # tmp
    mv a1, s5
    ret                 # return from eraseBuffer.section .text     
    ret
    li a2, 1
    
  andi  a1, a0, 1
    
    mv a0, s3
    lw t5, 48(sp)
    lw a5, 24(sp)
    
quicksort:
    li a3, 1 
    mv a2, s7
                    sw a2, 8(sp)
                sw a6, 0(a4) # p[i] = p[i+1]
    
    ret
    lw a7, 0(a4)    # load the value
    
    sw a5, 24(sp)
  addi  a0, a0, 1
    lw s4, 20(sp)
    # TODO: Return result of tallyNeighbors
        
        li s5, 32
    lw a4, 0(a2) # a4 holds the GPIO out value
    sw t0, 28(sp)
    add s3, s3, a0
    sw t2, 36(sp)


    slli a2, a2, 2  # a2: 4 * y for address 
    lw a1, 0(a2)        # a1: *GPIO_IN_ADDR
    blt t5, t3, skip
    add a2, a2, a0  # a2: screen_buffer + 4*y 
    lw s7, 32(sp)
        ret
# C declaration: void pinSetup(int pin_num, int mode)
    sw a5, 0(a4)   # store it back
    lw t1, 0(t0)   # ptr[j]
getPixel:
    addi a2, a2, 1      # increment i 


    or a7, a7, a6   # or with the prepared value
# ARGUMENTS a0: pin_num
    add t0, t0, a0 # &ptr[end]
    sw s0, 4(sp)
