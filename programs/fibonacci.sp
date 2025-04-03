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
mov $a1 0xF002
lw $a2 0xF000
mov $r0 0
sw $r0 $a1
add $a1 $a1 2
mov $r1 1
sw $r0 $a1
add $a1 $a1 2
sub $a2 2

@compute
add $r0 $r0 $r1 # Compute and store in r0
sw $r0 $a1      # Store in out
add $a1 $a1 2   # incr out
sub $a2 2       # decr counter
add $r2 $r1 0   # Swap r1 and r2
add $r1 $r0 0
add $r0 $r2 0
cmp $a2 0
jeq compute

mov $a2 0xFF
sw  $a2 0xFFFF
cmp $a2 0xFF
@end
jeq end