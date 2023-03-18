# a0: int* p
# a1: start
# a2: end
partition:
    slli t0, a2, 2 # t0 = end * 4
    add t0, t0, a0 # &ptr[end]
    lw a3, 0(t0) # pivot

    mv a4, a1        # j
    addi a5, a4, -1  # tmp
    mv a6, a5        # i

    j cmp
    loop:

    slli t0, a4, 2 # t0 = j * 4
    add t0, t0, a0 # &ptr[j]
    lw t1, 0(t0)   # ptr[j]

    bge a3, t1, skip # skip pivot end >= ptr[j]
    addi a6, a6, 1 # i++

    slli t2, a6, 2 # i * 4
    add t2, t2, a0 # &ptr[i]
    lw a5, 0(t2) # tmp = ptr[i]

    sw t1, 0(t2) # ptr[i] = ptr[j]
    sw a5, 0(t0) # ptr[j] = tmp

    skip:

    addi a4, a4, 1 # j++
    cmp:
    blt a4, a2, loop

    addi a6, a6, 1 # i++
    slli t0, a6, 2 # i * 4
    addi t0, t0, 0

    lw a5, 0(t0)
    sw a3, 0(t0)

    slli t0, a2, 2 # t0 = end * 4
    add t0, t0, a0 # &ptr[end]

    sw a5, 0(t0)

    mv a0, a6
    ret



# C declaration: int pinRead(int pin_num)
# ARGUMENTS a0: pin_num
# RETURNS bit read from GPIO pin 
pinRead:
    li a2, 0x6000403C   # GPIO_IN_ADDR
    lw a1, 0(a2)        # a1: *GPIO_IN_ADDR
    srl a1, a1, a0      # shift down by pin_num
    andi a0, a1, 0x1
    ret


# C declaration: void pinWrite(int pin_num, int value)
# ARGUMENTS a0: pin_num, a1: value
# RETURNS: Nothing
pinWrite:
    li a2, 0x60004004   # GPIO_OUT_ADDR
    beqz a1, write_zero # check if value is 0
    
    # Now we write 1 to the pin
    lw a4, 0(a2) # a4 holds the GPIO out value
    li a3, 1 
    sll a3, a3, a0 # a3: 1 << pin_num
    or a4, a4, a3  # or the GPIO out value with 1 << pin_num
    sw a4, 0(a2)   # store it back
    ret

# writing 0 to a ping
write_zero:
    lw a4, 0(a2) # a4 holds the GPIO out value
    li a3, 1 
    sll a3, a3, a0 # a3: 1 << pin_num
    not a3, a3     # a3 = ~a3
    and a4, a4, a3 
    sw a4, 0(a2)
    ret



# C declaration: void pinSetup(int pin_num, int mode)
# ARGUMENTS a0: pin_num, a1: mode
# RETURNS: Nothing
pinSetup:
    li a2, 0x60009004   # IO_MUX_GPIOn_ADDR
    li a3, 0x60004020   # GPIO_ENABLE_ADDR
    beqz a1, config_input # handle mode = 0
    
    # Handle mode = 1
    li a4, 1
    sll a5, a4, a0 # a5 = 1 << pin_num
    lw a6, 0(a3) # load value in GPIO_ENABLE_ADDR
    or a6, a6, a5 # or with 1 << pin_num
    sw a6, 0(a3)
    ret
    
    
config_input:
    li a4, 2
    sll a4, a0, a4  # get the offset, put it in a4
    add a4, a4, a2  # add the offset, a4: ptr to write to
    
    li a5, 1        # just store 1
    slli a6, a5, 9  # a6 = 1 << 9
    slli a5, a5, 8  # a5 = 1 << 8
    add a6, a6, a5  # a6 = a6 + a5
    
    lw a7, 0(a4)    # load the value
    or a7, a7, a6   # or with the prepared value
    sw a7, 0(a4)    # load it back
    ret
 


# C declaration: void setPixel(uint32_t* screen_buffer_addr, uint8_t x, uint8_t y, uint8_t val)
# ARGUMENTS a0: screen buffer starting address, a1: x, a2: y, a3: val
# RETURNS: Nothing
setPixel:
    # First index correct integer
    slli a4, a2, 2 # calculating pointer offset: y << 2 = y * 2 
    add a4, a4, a0 # add it to the pointer to the array
    lw a5, 0(a4)   # read from it to get the correct
    
    beqz a3, set_pixel_zero
    
    li a6, 1
    sll a6, a6, a1  # a6: 1 << x
    or a5, a5, a6   # or a6 into a5 to set the bit
    sw a5, 0(a4)    # write it back
    ret
    
set_pixel_zero:
    li a6, 1
    sll a6, a6, a1 # a6: 1 << x
    not a6, a6     # not it so we can zero one bit
    and a5, a5, a6 # and it in
    sw a5, 0(a4)   # store it back
    ret


# C declaration: void eraseBuffer(uint32_t* screen_buffer_addr)
# ARGUMENTS a0: screen_buffer starting address
# RETURNS: Nothing
eraseBuffer:
    li a1, 8            # upper bound on for loop
    li a2, 0            # 'i' for for loop
looping: 
    slli a3, a2, 2      # calculate 4*i
    add a4, a0, a3      # get address of array element by adding base address + 4*i 
    sw zero, 0(a4)      # write 0 to memory address
    addi a2, a2, 1      # increment i 
    bne a2, a1, looping # continue looping if i < 8
    ret                 # return from eraseBuffer.section .text     

# updateBoard
# ARGUMENTS a0: screen buffer (current board), a1: temporary output buffer (for new board)
# RETURNS: Nothing
# updateBoard
# ARGUMENTS a0: screen buffer (current board),
# a1: temporary output buffer (for new board)
# RETURNS: Nothing
updateBoard:
    addi sp, sp, -36
    sw ra, 0(sp)
    sw s0, 4(sp)
    sw s1, 8(sp)
    sw s2, 12(sp)
    sw s3, 16(sp)
    sw s4, 20(sp)
    sw s5, 24(sp)
    sw s6, 28(sp)
    sw s7, 32(sp)

    li s1, 0 # x
    li s2, 0 # y
    
    mv s3, a0 # current_board
    mv s4, a1 # new_board
    
    mv a0, s4
    call eraseBuffer
    
    j outer_cmp
    outer:
        mv s1, zero # x = 0
        j inner_cmp # TODO: might not need
        inner:
            mv a0, s3
            mv a1, s1
            mv a2, s2
            call checkNeighbors
            mv s6, a0 # s6 now holds tally

            mv a0, s3
            mv a1, s1
            mv a2, s2
            call getPixel
            mv s7, a0 # s7 now holds pixel value

            li s5, 3
            bne s6, s5, two_neighbors # skip if not 3 neighbors
                mv a0, s4
                mv a1, s1
                mv a2, s2
                li a3, 1
                call setPixel
                j over

            two_neighbors:
            beqz s7, over # skip if cell is dead
                li s5, 2
                bne s6, s5, over # skip if not 2 neighbors
                mv a0, s4
                mv a1, s1
                mv a2, s2
                li a3, 1
                call setPixel

            over:

        addi s1, s1, 1 # x++
        inner_cmp:
        li s5, 32
        blt s1, s5, inner

    addi s2, s2, 1 # y++
    outer_cmp:
        li s5, 8
        blt s2, s5, outer
        
    lw ra, 0(sp)
    lw s0, 4(sp)
    lw s1, 8(sp)
    lw s2, 12(sp)
    lw s3, 16(sp)
    lw s4, 20(sp)
    lw s5, 24(sp)
    lw s6, 28(sp)
    lw s7, 32(sp)
    addi sp, sp, 36
    
    ret


# getPixel
# ARGUMENTS a0: screen_buffer, a1: x, a2: y
# RETURNS: 1 if cell occupied, 0 otherwise
getPixel:
    # your code here
    slli a2, a2, 2  # a2: 4 * y for address 
    add a2, a2, a0  # a2: screen_buffer + 4*y 
    lw a3, 0(a2)    # a3: screen_buffer[y]
    srl a3, a3, a1  # a3: screen_buffer[y] >>_l x
    andi a0, a3, 1  # a0: a3 & 1
    ret


# checkNeighbors
# ARGUMENTS a0: game_board, a1: x index, a2: y index
# RETURNS: total occupied cells in the eight surrounding cells of (x,y) (game board wraps in x and y)
checkNeighbors:
    # TODO: Determine left, right, up, and down indices for neighbors.
    addi sp, sp, -4
    sw ra, 0(sp)
    addi a3, a1, 1 # left
    addi a4, a1, -1 # right
    addi a5, a2, -1 # up
    addi a6, a2, 1 # down
    
    # modding x's
    slli a3, a3, 27
    srli a3, a3, 27
    slli a4, a4, 27
    srli a4, a4, 27
    
    # modding y's
    slli a5, a5, 29
    srli a5, a5, 29
    slli a6, a6, 29
    srli a6, a6, 29

    call tallyNeighbors # tallyNeighbors(game_board, x, y, left, right, up, down)
    lw ra, 0(sp)
    addi sp, sp, 4
    # TODO: Return result of tallyNeighbors
    
    ret

# tallyNeighbors
# ARGUMENTS a0: game_board, a1: current x index, a2: current y index,
# a3: left neighbor index, a4: right neighbor index, a5: up neighbor index,
# a6: down neighbor index
# RETURNS: total occupied cells in the eight surrounding cells of current (x,y)
tallyNeighbors:






    # TODO: This procedure is functionally correct, but doesn't follow
    # RISC-V calling convention.
    # Make this procedure follow calling convention. You may only add
    # instructions that:
    # 1. Increment/decrement the stack pointer
    # 2. Put elements on the stack
    # 3. Take elements off the stack
    
    addi sp, sp, -36
    sw ra, 0(sp)
    sw s0, 4(sp)
    sw s1, 8(sp)
    sw s2, 12(sp)
    sw s3, 16(sp)
    sw s4, 20(sp)
    sw s5, 24(sp)
    sw s6, 28(sp)
    sw s7, 32(sp)

    mv s0, a0 # s0: game_board
    mv s1, a1 # a1: x
    mv s2, a2 # a2: y
    
    li s3, 0  # s3: tally
    
    # store a registers in s registers
    mv s4, a3 # left
    mv s5, a4 # right
    mv s6, a5 # up
    mv s7, a6 # down

    mv a1, s5
    mv a2, s6
    call getPixel # getPixel(game_board, x-1, y-1)
    add s3, s3, a0

    mv a0, s0
    mv a1, s5
    mv a2, s2
    call getPixel # getPixel(game_board, x-1, y)
    add s3, s3, a0

    mv a0, s0
    mv a1, s5
    mv a2, s7
    call getPixel # getPixel(game_board, x-1, y+1)
    add s3, s3, a0

    mv a0, s0
    mv a1, s1
    mv a2, s6
    call getPixel # getPixel(game_board, x, y-1)
    add s3, s3, a0

    mv a0, s0
    mv a1, s1
    mv a2, s7
    call getPixel # getPixel(game_board, x, y+1)
    add s3, s3, a0

    mv a0, s0
    mv a1, s4
    mv a2, s6
    call getPixel # getPixel(game_board, x+1, y-1)
    add s3, s3, a0

    mv a0, s0
    mv a1, s4
    mv a2, s2
    call getPixel # getPixel(game_board, x+1, y)
    add s3, s3, a0

    mv a0, s0
    mv a1, s4
    mv a2, s7
    call getPixel # getPixel(game_board, x+1, y+1)
    add s3, s3, a0

    mv a0, s3
    
    lw ra, 0(sp)
    lw s0, 4(sp)
    lw s1, 8(sp)
    lw s2, 12(sp)
    lw s3, 16(sp)
    lw s4, 20(sp)
    lw s5, 24(sp)
    lw s6, 28(sp)
    lw s7, 32(sp)
    addi sp, sp, 36

    ret

collatz:
If:
  andi  a1, a0, 1
  bnez  a1, Else
Then:
  srai  a0, a0, 1
  j     End
Else:
  slli  a2, a0, 1
  add   a0, a2, a0
  addi  a0, a0, 1
End:
  ret
# C declaration: int pinRead(int pin_num)
# ARGUMENTS a0: pin_num
# RETURNS bit read from GPIO pin 
pinRead:
    li a2, 0x6000403C   # GPIO_IN_ADDR
    lw a1, 0(a2)        # a1: *GPIO_IN_ADDR
    srl a1, a1, a0      # shift down by pin_num
    andi a0, a1, 0x1
    ret


# C declaration: void pinWrite(int pin_num, int value)
# ARGUMENTS a0: pin_num, a1: value
# RETURNS: Nothing
pinWrite:
    li a2, 0x60004004   # GPIO_OUT_ADDR
    beqz a1, write_zero # check if value is 0
    
    # Now we write 1 to the pin
    lw a4, 0(a2) # a4 holds the GPIO out value
    li a3, 1 
    sll a3, a3, a0 # a3: 1 << pin_num
    or a4, a4, a3  # or the GPIO out value with 1 << pin_num
    sw a4, 0(a2)   # store it back
    ret

# writing 0 to a ping
write_zero:
    lw a4, 0(a2) # a4 holds the GPIO out value
    li a3, 1 
    sll a3, a3, a0 # a3: 1 << pin_num
    not a3, a3     # a3 = ~a3
    and a4, a4, a3 
    sw a4, 0(a2)
    ret



# C declaration: void pinSetup(int pin_num, int mode)
# ARGUMENTS a0: pin_num, a1: mode
# RETURNS: Nothing
pinSetup:
    li a2, 0x60009004   # IO_MUX_GPIOn_ADDR
    li a3, 0x60004020   # GPIO_ENABLE_ADDR
    beqz a1, config_input # handle mode = 0
    
    # Handle mode = 1
    li a4, 1
    sll a5, a4, a0 # a5 = 1 << pin_num
    lw a6, 0(a3) # load value in GPIO_ENABLE_ADDR
    or a6, a6, a5 # or with 1 << pin_num
    sw a6, 0(a3)
    ret
    
    
config_input:
    li a4, 2
    sll a4, a0, a4  # get the offset, put it in a4
    add a4, a4, a2  # add the offset, a4: ptr to write to
    
    li a5, 1        # just store 1
    slli a6, a5, 9  # a6 = 1 << 9
    slli a5, a5, 8  # a5 = 1 << 8
    add a6, a6, a5  # a6 = a6 + a5
    
    lw a7, 0(a4)    # load the value
    or a7, a7, a6   # or with the prepared value
    sw a7, 0(a4)    # load it back
    ret
 


# C declaration: void setPixel(uint32_t* screen_buffer_addr, uint8_t x, uint8_t y, uint8_t val)
# ARGUMENTS a0: screen buffer starting address, a1: x, a2: y, a3: val
# RETURNS: Nothing
setPixel:
    # First index correct integer
    slli a4, a2, 2 # calculating pointer offset: y << 2 = y * 2 
    add a4, a4, a0 # add it to the pointer to the array
    lw a5, 0(a4)   # read from it to get the correct
    
    beqz a3, set_pixel_zero
    
    li a6, 1
    sll a6, a6, a1  # a6: 1 << x
    or a5, a5, a6   # or a6 into a5 to set the bit
    sw a5, 0(a4)    # write it back
    ret
    
set_pixel_zero:
    li a6, 1
    sll a6, a6, a1 # a6: 1 << x
    not a6, a6     # not it so we can zero one bit
    and a5, a5, a6 # and it in
    sw a5, 0(a4)   # store it back
    ret


# C declaration: void eraseBuffer(uint32_t* screen_buffer_addr)
# ARGUMENTS a0: screen_buffer starting address
# RETURNS: Nothing
eraseBuffer:
    li a1, 8            # upper bound on for loop
    li a2, 0            # 'i' for for loop
looping: 
    slli a3, a2, 2      # calculate 4*i
    add a4, a0, a3      # get address of array element by adding base address + 4*i 
    sw zero, 0(a4)      # write 0 to memory address
    addi a2, a2, 1      # increment i 
    bne a2, a1, looping # continue looping if i < 8
    ret                 # return from eraseBuffer.section .text     

quicksort:
    bge a1, a2, end # end if start >= end
    addi sp, sp, -28
    sw ra, 0(sp)  # save ra
    sw a0, 4(sp)  # save p
    sw a1, 8(sp)  # save start
    sw a2, 12(sp) # save end
    sw s0, 16(sp)
    sw s1, 20(sp)
    sw s2, 24(sp)
    
    call partition
    mv s0, a0 # s0 stores q
    addi s1, s0, 1  # q + 1
    addi s2, s0, -1 # q - 1
    
    lw a0, 4(sp)
    lw a1, 8(sp)
    mv a2, s2
    call quicksort
    
    lw a0, 4(sp)
    mv a1, s1
    lw a2, 12(sp)
    call quicksort
        
    lw ra, 0(sp)
    lw a0, 4(sp)
    lw a1, 8(sp)
    lw a2, 12(sp)
    lw s0, 16(sp)
    lw s1, 20(sp)
    lw s2, 24(sp)
    addi sp, sp, 28
    
    end:
        ret


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

    // >>>>>>>>>>>>>>>>>>>>>>>>>
    sw a0, 4(sp)
    sw a1, 8(sp)
    sw a2, 12(sp)
    sw a3, 16(sp)
    sw a4, 24(sp)
    sw a5, 24(sp)
    sw t0, 28(sp)
    sw t1, 32(sp)
    sw t2, 36(sp)
    sw t3, 40(sp)
    sw t4, 44(sp)
    sw t5, 48(sp)
    call arrayViz
    lw a0, 4(sp)
    lw a1, 8(sp)
    lw a2, 12(sp)
    lw a3, 16(sp)
    lw a4, 20(sp)
    lw a5, 24(sp)
    lw t0, 28(sp)
    lw t1, 32(sp)
    lw t2, 36(sp)
    lw t3, 40(sp)
    lw t4, 44(sp)
    lw t5, 48(sp)
    // >>>>>>>>>>>>>>>>>>>>>>>>>

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

    // >>>>>>>>>>>>>>>>>>>>>>>>>
    sw a0, 4(sp)
    sw a1, 8(sp)
    sw a2, 12(sp)
    sw a3, 16(sp)
    sw a4, 24(sp)
    sw a5, 24(sp)
    sw t0, 28(sp)
    sw t1, 32(sp)
    sw t2, 36(sp)
    sw t3, 40(sp)
    sw t4, 44(sp)
    sw t5, 48(sp)
    call arrayViz
    lw a0, 4(sp)
    lw a1, 8(sp)
    lw a2, 12(sp)
    lw a3, 16(sp)
    lw a4, 20(sp)
    lw a5, 24(sp)
    lw t0, 28(sp)
    lw t1, 32(sp)
    lw t2, 36(sp)
    lw t3, 40(sp)
    lw t4, 44(sp)
    lw t5, 48(sp)
    // >>>>>>>>>>>>>>>>>>>>>>>>>

    mv a0, a5

    lw ra, 0(sp)
    addi sp, sp, 52
 
    ret




  

   

  


bubblesort:
    li a2, 1
    addi a1, a1, -1
    
while:
    li a2, 0 # swapped = 0
    li a3, 0 # i = 0
        for:
        bge a3, a1, end_for # end the for loop if i >= n
            slli a4, a3, 2 # offset = i << 2 = i * 4
            add a4, a4, a0 # a4 now stores ptr + offset
            lw a5, 0(a4) # p[i]
            lw a6, 4(a4) # p[i+1]
            bge a6, a5, noswap # go to noswap
                # doing the swap
                mv a7, a5 # tmp = p[i]
                sw a6, 0(a4) # p[i] = p[i+1]
                sw a7, 4(a4) # p[i+1] = p[i]
                li a2, 1 # swapped = 1
                
                # stuff for printing
                    addi sp, sp, -36
                    sw a0, 0(sp)
                    sw a1, 4(sp)
                    sw a2, 8(sp)
                    sw a3, 12(sp)
                    sw a4, 16(sp)
                    sw a5, 20(sp)
                    sw a6, 24(sp)
                    sw a7, 28(sp)
                    sw ra, 32(sp)
                    jal arrayViz
                    lw a0, 0(sp)
                    lw a1, 4(sp)
                    lw a2, 8(sp)
                    lw a3, 12(sp)
                    lw a4, 16(sp)
                    lw a5, 20(sp)
                    lw a6, 24(sp)
                    lw a7, 28(sp)
                    lw ra, 32(sp)
                    addi sp, sp, 36
            noswap:
            addi a3, a3, 1 # i++
            j for # go to top of for loop
        end_for:
    addi a1, a1, -1 # n--
    bnez a2, while # go to top of while loop