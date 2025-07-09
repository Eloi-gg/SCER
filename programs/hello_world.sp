!display_enable 0b10000000
!display_write 0b01000000
!display_clear 0b00100000
!display_cursor_move 0b00010000
!cursor_mask 0b00001100
!cursor_up 0b00001000
!cursor_down 0b00000000
!cursor_left 0b00000100
!cursor_right 0b00001100
!addr_ctrl 0xC000
!addr_data 0xC001
!test_end_address 0xFFFF

mov $r0 display_enable
or $r0 $r0 display_write

mov $r1 'h

sw $r1 addr_data
sw $r0 addr_ctrl



# program end
mov $a2 0xFF    
sw  $a2 test_end_address