# Arithmetic immediate

add $a0 $r1 1
sub $a1 $r0 255
and $a2 $a2 0xFF
or $r0 $f 12345
xor $r1 $a2 0xFFFF
asl $r2 $a0 1
asr $z $z 4
cmp $a0 $f 0

# Arithmetic register

add $a0 $r1 $r2
sub $a1 $r0 $r2
and $a2 $a2 $r0
or $r0 $f $r2
xor $r1 $a2 $r0
asl $r2 $a0 $r1
asr $z $z $r2
cmp $a0 $f $r2

# ....

# Labels