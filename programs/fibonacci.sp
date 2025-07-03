# Fibonacci sequence:
# 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, ...
# U0 = 0
# U1 = 1
# Un = Un-1 + Un-2

# a1 = storing address
# a2 = number of elements to compute

# 0xF000 = input (number of elements)
# 0xF002 = output (address of the first element)
# 0xF004 = address of the second element
# ....
# 0xFFFF = set to high to indicate test end

!input_address 0xF000
!output_address 0xF002
!test_end_address 0xFFFF

@start
mov $a1 output_address
lw $a2 input_address
mov $r0 0
sw $r0 $a1
add $a1 $a1 2
mov $r1 1
sw $r1 $a1
add $a1 $a1 2
sub $a2 $a2 2

@compute
add $r0 $r0 $r1 # Compute and store in r0
sw $r0 $a1      # Store in out
add $a1 $a1 2   # incr out
sub $a2 $a2 1   # decr counter
add $r2 $r1 0   # Swap r1 and r2
add $r1 $r0 0
add $r0 $r2 0
cmp $a2 0
jne compute

mov $a2 0xFF    # load a2 with 1's
sw  $a2 test_end_address  # store 1's in test end address to indicate test end
cmp $a2 0xFF    # reset comparison register
@end            # infinite loop
jeq end