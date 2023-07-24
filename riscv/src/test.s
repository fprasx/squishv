pinRead:
    add s3, s3, a0
    lw a2, 12(sp)
  
pinWrite:
 
    blt a4, a2, loop
    mv a1, s4
    li a2, 0x60009004   # IO_MUX_GPIOn_ADDR
    bnez a2, while # go to top of while loop
    lw t0, 28(sp)
# RETURNS: Nothing
# ARGUMENTS a0: screen buffer (current board), a1: temporary output buffer (for new board)
tallyNeighbors:
    slli a4, a4, 27
            mv s6, a0 # s6 now holds tally
    sw a5, 0(a4)    # write it back
    andi a0, a1, 0x1
    sw a7, 0(a4)    # load it back
    ret
    sw ra, 0(sp)
# C declaration: int pinRead(int pin_num)
    li a3, 0x60004020   # GPIO_ENABLE_ADDR

    slli a2, a2, 2  # a2: 4 * y for address 
    slli t2, a6, 2 # i * 4
    
    slli a4, a2, 2 # calculating pointer offset: y << 2 = y * 2 
    outer_cmp:
    lw s0, 4(sp)
    sw t4, 44(sp)
    lw a1, 8(sp)
        end_for:
# RETURNS: total occupied cells in the eight surrounding cells of (x,y) (game board wraps in x and y)
    # modding x's
    skip:
    sw zero, 0(a4)      # write 0 to memory address
    add s3, s3, a0
    # Make this procedure follow calling convention. You may only add
    sw a4, 0(t4)   # p[end] = temp
    lw s6, 28(sp)
    li a5, 1        # just store 1
# updateBoard
    lw ra, 0(sp)
# ARGUMENTS a0: pin_num, a1: value
    
    mv s4, a1 # new_board
    lw s4, 20(sp)
    and a4, a4, a3 
    slli a4, a2, 2 # calculating pointer offset: y << 2 = y * 2 
    lw t3, 40(sp)
    lw t2, 36(sp)
    sw a0, 4(sp)
    add a4, a4, a2  # add the offset, a4: ptr to write to
            lw a5, 0(a4) # p[i]
            addi a3, a3, 1 # i++
    bne a2, a1, looping # continue looping if i < 8
    sll a6, a6, a1 # a6: 1 << x

                    lw a1, 4(sp)
            bge a6, a5, noswap # go to noswap
    srli a4, a4, 27
    ret
    sw s2, 12(sp)
    sw a5, 0(a4)   # store it back

    call quicksort
    li a2, 0x60009004   # IO_MUX_GPIOn_ADDR
eraseBuffer:
    lw a6, 0(a3) # load value in GPIO_ENABLE_ADDR
    li a3, 0 # i = 0
    li a3, 1 
    li a6, 1
# ARGUMENTS a0: game_board, a1: current x index, a2: current y index,

            noswap:
# RETURNS: total occupied cells in the eight surrounding cells of current (x,y)

    mv s0, a0 # s0 stores q
    # 3. Take elements off the stack
    sw a7, 0(a4)    # load it back
    mv a2, s6
    and a5, a5, a6 # and it in

    not a6, a6     # not it so we can zero one bit
                    sw a1, 4(sp)
    sll a5, a4, a0 # a5 = 1 << pin_num
    addi a5, a4, -1  # tmp

    
    lw s5, 24(sp)
    mv a2, s2
    slli a3, a2, 2      # calculate 4*i
# RETURNS: Nothing
    sw a3, 16(sp)
    li a2, 0x60004004   # GPIO_OUT_ADDR
while:
    
    // >>>>>>>>>>>>>>>>>>>>>>>>>
        bge a3, a1, end_for # end the for loop if i >= n
    # TODO: This procedure is functionally correct, but doesn't follow
    sw t5, 48(sp)
                li s5, 2
    addi sp, sp, -52
    
    lw ra, 0(sp)
    addi a6, a6, 1 # i++
    add a4, a4, a0 # add it to the pointer to the array
    ret
    lw a0, 4(sp)
pinSetup:
    sw s6, 28(sp)
    beqz a1, write_zero # check if value is 0
        ret
    li a5, 1        # just store 1
    li a2, 0 # swapped = 0
    lw t5, 48(sp)
    add s3, s3, a0
    sw a6, 0(a3)
# C declaration: void pinSetup(int pin_num, int mode)
    cmp:
                # doing the swap
# updateBoard
    addi sp, sp, -36
    call getPixel # getPixel(game_board, x+1, y+1)
    lw t4, 44(sp)
                    sw a2, 8(sp)
    mv a0, s0
    j cmp

    addi a4, a4, 1 # j++
# RETURNS bit read from GPIO pin 
    sw t5, 0(t0)   # p[i] = end
    lw a3, 0(t0) # pivot
    sw s6, 28(sp)
    mv a2, s6
                
    ret
    slli t0, a4, 2 # t0 = j * 4
    sw a0, 4(sp)
    ret                 # return from eraseBuffer.section .text     
    sll a4, a0, a4  # get the offset, put it in a4
    call getPixel # getPixel(game_board, x-1, y-1)
                li a3, 1
# a0: int* p
    slli a6, a6, 29
                    sw a7, 28(sp)
    sw t3, 40(sp)
    lw a2, 12(sp)
# ARGUMENTS a0: game_board, a1: x index, a2: y index

    lw a4, 0(a2) # a4 holds the GPIO out value
quicksort:
# writing 0 to a ping
    blt a3, a2, loop
    add t0, t0, a0 # &ptr[end]
    sll a6, a6, a1 # a6: 1 << x
    lw s4, 20(sp)
                # stuff for printing

    sw ra, 0(sp)
    addi a4, a1, -1 # right
    li a6, 1
        blt s2, s5, outer
    sw s0, 4(sp)
    loop:
    
    lw a5, 24(sp)

            li s5, 3
pinWrite:
# RETURNS: Nothing
    li a3, 1 
    slli a6, a5, 9  # a6 = 1 << 9
# C declaration: void setPixel(uint32_t* screen_buffer_addr, uint8_t x, uint8_t y, uint8_t val)
    # First index correct integer
    lw s1, 20(sp)
    sw a3, 16(sp)
    addi a2, a2, 1      # increment i 
pinSetup:
    sw a4, 24(sp)
    mv a2, s7


        
    addi a3, a3, 1 # j++
    li a2, 0x6000403C   # GPIO_IN_ADDR
    lw a3, 16(sp)
    li a4, 1
    mv a0, a6
    
                mv a1, s1
                    sw a3, 12(sp)

    add t2, t2, a0 # j offset
                mv a1, s1
    li s3, 0  # s3: tally
    or a4, a4, a3  # or the GPIO out value with 1 << pin_num

checkNeighbors:

collatz:
    outer:

    sw s5, 24(sp)

    mv a0, s0
    # Now we write 1 to the pin

    sw a4, 24(sp)
    not a3, a3     # a3 = ~a3
            mv s7, a0 # s7 now holds pixel value
    sw t0, 28(sp)

    lw a1, 8(sp)
                j over
            add a4, a4, a0 # a4 now stores ptr + offset
    lw t1, 32(sp)
    slli t2, a3, 2 # j * 4

    # Handle mode = 1
                    lw ra, 32(sp)
    mv s7, a6 # down
Else:

    bge a1, a2, end # end if start >= end
    lw a2, 12(sp)
partition:
            two_neighbors:
# ARGUMENTS a0: pin_num, a1: mode
    sw a6, 0(a3)
  srai  a0, a0, 1
    
    or a6, a6, a5 # or with 1 << pin_num
        li s5, 8

    mv a1, s4
    loop:
    sw a5, 24(sp)

    sw a1, 8(sp)
            call checkNeighbors
    li a2, 1
# C declaration: void setPixel(uint32_t* screen_buffer_addr, uint8_t x, uint8_t y, uint8_t val)
    # Handle mode = 1
    lw s2, 12(sp)
    lw s2, 24(sp)
    addi sp, sp, -4
    sll a3, a3, a0 # a3: 1 << pin_num
    ret
    mv a1, s5
    li a2, 0x6000403C   # GPIO_IN_ADDR
    sw a2, 12(sp) # save end
                    sw a4, 16(sp)
# RETURNS: Nothing

                mv a0, s4


# ARGUMENTS a0: screen_buffer, a1: x, a2: y
    # store a registers in s registers
    mv a0, s0
    lw a5, 0(a4)   # read from it to get the correct
    mv a1, s1

  addi  a0, a0, 1
    # >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
    add s3, s3, a0
    li a6, 1
    // >>>>>>>>>>>>>>>>>>>>>>>>>
        blt s1, s5, inner

looping: 
updateBoard:
    or a7, a7, a6   # or with the prepared value
    mv a0, s3
    li a4, 2
    sw t1, 32(sp)
    lw ra, 0(sp)
    sw a5, 0(a4)   # store it back
    li a2, 0            # 'i' for for loop
    add a4, a0, a3      # get address of array element by adding base address + 4*i 

# C declaration: void eraseBuffer(uint32_t* screen_buffer_addr)
# RETURNS: Nothing
                mv a7, a5 # tmp = p[i]
    slli t0, a6, 2 # i * 4
    lw a4, 20(sp)
    # instructions that:
        mv s1, zero # x = 0
    sw zero, 0(a4)      # write 0 to memory address
    // >>>>>>>>>>>>>>>>>>>>>>>>>
            call getPixel
    li a3, 1 
    addi t0, t0, 0
config_input:
# ARGUMENTS a0: screen_buffer starting address

    add t2, t2, a0 # &ptr[i]
                bne s6, s5, over # skip if not 2 neighbors
                    sw a0, 0(sp)
            mv a1, s1
# a0: int* p
    lw a0, 4(sp)
    sw ra, 0(sp)
    ret
    lw a5, 24(sp)
    slli a5, a5, 8  # a5 = 1 << 8
    add a6, a6, a5  # a6 = a6 + a5
    mv a0, s0
    mv a0, a5
    addi a5, a5, 1 # i++
    
    lw t1, 32(sp)
    
            mv a2, s2

    lw s3, 16(sp)
    mv a1, s4
    lw s1, 8(sp)

    mv s6, a5 # up
    sll a6, a6, a1  # a6: 1 << x
    or a7, a7, a6   # or with the prepared value

    lw s3, 16(sp)
    call partition
    ret
    mv s2, a2 # a2: y
    lw a1, 8(sp)
    lw a0, 4(sp)
    or a5, a5, a6   # or a6 into a5 to set the bit
    lw t5, 0(t4) # pivot
    slli a5, a5, 8  # a5 = 1 << 8
    sw a3, 0(t0)
        li s5, 32
    lw a5, 0(a4)   # read from it to get the correct
    end:

    add a4, a0, a3      # get address of array element by adding base address + 4*i 
            mv a1, s1
    mv a1, s1

    ret                 # return from eraseBuffer.section .text     
                    lw a4, 16(sp)
# RETURNS: 1 if cell occupied, 0 otherwise
    li a3, 1 
                call setPixel
    
    
    li a1, 8            # upper bound on for loop
 

    
    addi sp, sp, 4

  ret
    sw a5, 0(t0) # ptr[j] = tmp
    ret
    mv a6, a5        # i

    ret
    addi sp, sp, 28
# getPixel
setPixel:


        inner_cmp:
    sw s3, 16(sp)
    
    ret
    
    ret

    slli t0, a2, 2 # t0 = end * 4
    mv s1, a1 # a1: x
# a2: end

    sll a5, a4, a0 # a5 = 1 << pin_num
    j cmp

    mv a0, s0
    # RISC-V calling convention.
set_pixel_zero:
    # 1. Increment/decrement the stack pointer
setPixel:
    lw a1, 0(a2)        # a1: *GPIO_IN_ADDR
    lw t2, 36(sp)
    li s2, 0 # y
    
            mv a2, s2
    ret
    sw t3, 0(t0) # p[i] = p[j]
    lw s0, 4(sp)
            lw a6, 4(a4) # p[i+1]

    lw s7, 32(sp)
    sw a2, 12(sp)
    lw t4, 44(sp)
    lw a5, 0(t0)
    addi sp, sp, 36
# RETURNS: Nothing
    lw a3, 16(sp)
    lw a1, 0(a2)        # a1: *GPIO_IN_ADDR
    srl a3, a3, a1  # a3: screen_buffer[y] >>_l x
    sw s2, 24(sp)
    addi s2, s2, 1 # y++
    addi sp, sp, -28
    lw a4, 0(a2) # a4 holds the GPIO out value
    sw a5, 24(sp)
    add t0, t0, a0 # i offset
            bne s6, s5, two_neighbors # skip if not 3 neighbors
looping: 

    add t0, t0, a0 # &ptr[j]
    sw s1, 8(sp)
  add   a0, a2, a0
                mv a0, s4
                sw a7, 4(a4) # p[i+1] = p[i]

    j outer_cmp
                mv a2, s2
    mv a2, s7
    add s3, s3, a0
        j inner_cmp # TODO: might not need
    
    mv a4, a1        # j
    lw s0, 16(sp)
    sw t4, 44(sp)
    mv a0, s4
    # >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
# tallyNeighbors
                    lw a3, 12(sp)
    li s1, 0 # x
    sw a1, 8(sp)  # save start
                    lw a2, 8(sp)
                    jal arrayViz
    skip:
                    lw a0, 0(sp)
    # Now we write 1 to the pin
# ARGUMENTS a0: pin_num
    call arrayViz
    beqz a3, set_pixel_zero
    ret

# C declaration: void pinWrite(int pin_num, int value)
    

    li a2, 0            # 'i' for for loop
    lw a1, 8(sp)
# C declaration: void eraseBuffer(uint32_t* screen_buffer_addr)
        for:
    mv a2, s6
# writing 0 to a ping
    ret
    addi a3, a1, 1 # left
    beqz a3, set_pixel_zero

        
    # TODO: Determine left, right, up, and down indices for neighbors.
# ARGUMENTS a0: pin_num, a1: mode
    # 2. Put elements on the stack
# a1: start

                    lw a5, 20(sp)
Then:
    
    add s3, s3, a0
  bnez  a1, Else

            beqz s7, over # skip if cell is dead
    sw s5, 24(sp)
    srli a5, a5, 29
   
    add s3, s3, a0
    lw a0, 4(sp)
    add t0, t0, a0 # i offset
    addi a1, a1, -1 # n--
    addi a1, a1, -1
    addi sp, sp, 52
    slli t0, a2, 2 # t0 = end * 4
    andi a0, a3, 1  # a0: a3 & 1
# checkNeighbors

    mv s3, a0 # current_board
    
                    addi sp, sp, -36
    lw a2, 12(sp)
    call getPixel # getPixel(game_board, x+1, y)
    // >>>>>>>>>>>>>>>>>>>>>>>>>
    ret
    call tallyNeighbors # tallyNeighbors(game_board, x, y, left, right, up, down)
    sw a4, 0(t2) # p[j] = tmp
    slli t4, a2, 2 # t0 = end * 4
    bge a3, t1, skip # skip pivot end >= ptr[j]
    lw s5, 24(sp)
# a1: temporary output buffer (for new board)
    ret
# a2: end
    addi a4, a1, -1  # tmp
# RETURNS: Nothing

    li a1, 8            # upper bound on for loop
    sw a0, 4(sp)  # save p
    

                li a3, 1
    sw a4, 0(a2)
    or a4, a4, a3  # or the GPIO out value with 1 << pin_num
            mv a0, s3
    call arrayViz
                mv a2, s2
getPixel:
    sw s7, 32(sp)

    
    srli a6, a6, 29
    add t0, t0, a0 # &ptr[end]

    sw t5, 48(sp)
        inner:
    li a2, 0x60004004   # GPIO_OUT_ADDR
    sw ra, 0(sp)
  j     End
write_zero:
    addi s1, s0, 1  # q + 1
    lw t1, 0(t0)   # ptr[j]
    sw a4, 0(a2)
    sw s4, 20(sp)
    sw a5, 0(a4)    # write it back
        addi s1, s1, 1 # x++
    sw t2, 36(sp)
                call setPixel
    



    beqz a1, config_input # handle mode = 0
    lw t0, 28(sp)
    
    sw a4, 0(a2)   # store it back
            j for # go to top of for loop
 
    call getPixel # getPixel(game_board, x+1, y-1)
    sw t1, 32(sp)
    call getPixel # getPixel(game_board, x, y-1)
    lw s6, 28(sp)
    srli a3, a3, 27
    addi a2, a2, 1      # increment i 
                sw a6, 0(a4) # p[i] = p[i+1]
    sw a5, 0(t0)

# ARGUMENTS a0: screen buffer starting address, a1: x, a2: y, a3: val
    and a4, a4, a3 
    
    mv a2, s2
# a1: start
    beqz a1, write_zero # check if value is 0
    
    sw ra, 0(sp)  # save ra
    not a3, a3     # a3 = ~a3
    lw a4, 0(t0) # tmp = p[i]
# a6: down neighbor index
    ret
set_pixel_zero:
    add a4, a4, a0 # add it to the pointer to the array
    # TODO: Return result of tallyNeighbors

# a3: left neighbor index, a4: right neighbor index, a5: up neighbor index,
                    lw a6, 24(sp)
    sw s2, 12(sp)
    lw a0, 4(sp)
    lw s7, 32(sp)
    addi sp, sp, -36
    mv a1, s5
    srl a1, a1, a0      # shift down by pin_num
    slli a6, a5, 9  # a6 = 1 << 9
    mv a0, s0
    lw a6, 0(a3) # load value in GPIO_ENABLE_ADDR
    lw a5, 0(t2) # tmp = ptr[i]
    sw s0, 4(sp)
                    lw a7, 28(sp)
  
    or a6, a6, a5 # or with 1 << pin_num

    addi a5, a1, -1  # i    
    sw s1, 20(sp)
# ARGUMENTS a0: screen_buffer starting address
    add a6, a6, a5  # a6 = a6 + a5
    sll a4, a0, a4  # get the offset, put it in a4
    

    li a4, 1
# ARGUMENTS a0: screen buffer starting address, a1: x, a2: y, a3: val
    slli a3, a2, 2      # calculate 4*i
    lw a4, 0(t0)   # tmp = p[i]
    sw s7, 32(sp)
    addi a6, a6, 1 # i++
    lw t5, 48(sp)
    call getPixel # getPixel(game_board, x-1, y)
    add s3, s3, a0
    mv s5, a4 # right
    li a6, 1
    # modding y's
    lw s1, 8(sp)
If:
            mv a0, s3
    ret
End:

    add a4, a4, a2  # add the offset, a4: ptr to write to
    blt t5, t3, skip

    call getPixel # getPixel(game_board, x-1, y+1)
config_input:
    and a5, a5, a6 # and it in
    lw a3, 0(a2)    # a3: screen_buffer[y]
    lw a7, 0(a4)    # load the value
    lw a4, 20(sp)
    andi a0, a1, 0x1
partition:
    mv a1, s5
    lw t3, 0(t2)   # p[j]
    add t4, t4, a0 # &ptr[end]
    lw s2, 12(sp)
# RETURNS: Nothing

    mv s0, a0 # s0: game_board
    not a6, a6     # not it so we can zero one bit
    or a5, a5, a6   # or a6 into a5 to set the bit
                    sw a6, 24(sp)
    call eraseBuffer
    call getPixel # getPixel(game_board, x, y+1)

    sll a3, a3, a0 # a3: 1 << pin_num
    

    addi a5, a2, -1 # up
    sll a6, a6, a1  # a6: 1 << x
    
    beqz a1, config_input # handle mode = 0
    mv a0, s0
    li a4, 2
    
    sw s0, 16(sp)
                    sw ra, 32(sp)
                    addi sp, sp, 36
    mv a2, s7
    ret


                li a2, 1 # swapped = 1
eraseBuffer:

    srl a1, a1, a0      # shift down by pin_num
    sw a4, 0(a2)   # store it back
    addi a5, a5, 1 # i++
    addi a6, a2, 1 # down

# RETURNS: Nothing
    mv s4, a3 # left
# ARGUMENTS a0: screen buffer (current board),
    sw s4, 20(sp)
    addi sp, sp, 36
    slli a5, a5, 29
    sw a2, 12(sp)
    lw t3, 40(sp)
    lw a4, 0(a2) # a4 holds the GPIO out value

    
    bne a2, a1, looping # continue looping if i < 8
bubblesort:
    cmp:
    lw a7, 0(a4)    # load the value

    lw ra, 0(sp)
    addi s2, s0, -1 # q - 1
                    sw a5, 20(sp)
# C declaration: void pinSetup(int pin_num, int mode)
    li a3, 0x60004020   # GPIO_ENABLE_ADDR

    
    slli t0, a5, 2 # i * 4
    mv a2, s2
    sw s3, 16(sp)
    sw t2, 36(sp)
    # First index correct integer

    lw a4, 0(a2) # a4 holds the GPIO out value
write_zero:
    ret
    sw s1, 8(sp)
            slli a4, a3, 2 # offset = i << 2 = i * 4
    sll a3, a3, a0 # a3: 1 << pin_num
    add a2, a2, a0  # a2: screen_buffer + 4*y 
pinRead:
    

    sll a3, a3, a0 # a3: 1 << pin_num
            over:
    sw t3, 40(sp)

  slli  a2, a0, 1
    sw a1, 8(sp)
    mv a3, a1        # j
    
    

    slli a3, a3, 27
    sw t1, 0(t2) # ptr[i] = ptr[j]

    sw t0, 28(sp)
    
    slli t0, a5, 2 # i * 4
  andi  a1, a0, 1

    
    call quicksort
    
    # your code here
    

    
    lw ra, 0(sp)
    mv a1, s1
