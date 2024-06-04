# File generated using DiSPA
execute as @e[tag=test_file,tag=test] if score $test_file-test_file timer matches 0 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation:{translation: [0f,1f,0f]}}
execute as @e[tag=test_file,tag=test] if score $test_file-test_file timer matches 0 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation:{left_rotation: [0f,0.70710677f,0f,0.70710677f]}}
execute as @e[tag=test_file,tag=test] if score $test_file-test_file timer matches 0 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation:{scale: [2f,2f,2f]}}
execute as @e[tag=test_file,tag=test] at @s if score $test_file-test_file timer matches 40 run summon block_display ~ ~ ~ {Tags:["test_file","test2"]}

execute if score $test_file-test_file timer matches 40.. run scoreboard players set $test_file-test_file flags 0
execute if score $test_file-test_file timer matches 40.. run scoreboard players set $test_file-test_file timer -1

scoreboard players add $test_file-test_file timer 1