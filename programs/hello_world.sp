!display_enable 0b10000000
!display_write 0b01000000
!display_clear 0b00100000
!cursor_mask 0b00001100
!cursor_up 0b00001000
!cursor_down 0b00000000
!cursor_left 0b00000100
!cursor_right 0b00001100
!addr_ctrl 0xC000
!addr_data 0xC001
!test_end_address 0xFFFF

mov $r0 0
cmp $r0 0
jeq main

##################################
# desc:
## sends character to the console
## auto increments cursor position
# args: 
## r0: letter to print on screen
#############
@print_letter

sw $r0 addr_data
mov $r0 display_enable
or $r0 $r0 display_write # write mode
sw $r0 addr_ctrl # Send: print r0

mov $r0 cursor_right
or $r0 $r0 display_enable
sw $r0 addr_ctrl

mov $r0 0
sw $r0 addr_ctrl
cmp $r0 0
jeq $z
##################################

@main
push 'h
push 'e
push 'l
push 'l
push 'o
push '_
push 'w
push 'o
push 'r
push 'l
push 'd
push '!
mov $a0 12 # 12 chars

@print_loop
pop $r0
sub $a0 $a0 1
cmp $a0 0
jne print_letter
cmp $a0 0
jne print_loop
cmp $a0 0
jeq print_letter

# program end
@end
mov $a2 0xFF    
sw  $a2 test_end_address