# File generated using Dispaexecute as @e[tag=test_file-e1] if score $test_file timer matches 0 run data merge entity @s {start_interpolation:0,interpolation_duration:0,transformation: {translation: [0f,1f,0f]}}
execute as @e[tag=test_file-e1] if score $test_file timer matches 10 run data merge entity @s {start_interpolation:0,interpolation_duration:10,transformation: {translation: [0f,2f,0f]}}
execute as @e[tag=test_file-e1] if score $test_file timer matches 10 run data merge entity @s {start_interpolation:0,interpolation_duration:10,transformation: {left_rotation: [0.70710677f,0f,0.70710677f,0f]}}
execute as @e[tag=test_file-e1] if score $test_file timer matches 20 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation: {translation: [1f,0f,0f]}}
execute as @e[tag=test_file-e1] if score $test_file timer matches 20 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation: {left_rotation: [0.70710677f,0.70710677f,0f,0f]}}
execute as @e[tag=test_file-e1] if score $test_file timer matches 30 run data merge entity @s {start_interpolation:0,interpolation_duration:30,transformation: {translation: [2f,0f,0f]}}
execute as @e[tag=test_file-e1] if score $test_file timer matches 30 run data merge entity @s {start_interpolation:0,interpolation_duration:30,transformation: {translation: [2f,0f,1f]}}
execute as @e[tag=test_file-e1] if score $test_file timer matches 30 run data merge entity @s {start_interpolation:0,interpolation_duration:30,transformation: {left_rotation: [0.49999997f,0.49999997f,0.49999997f,-0.49999997f]}}
execute as @e[tag=test_file-e1] if score $test_file timer matches 30 run data merge entity @s {start_interpolation:0,interpolation_duration:30,transformation: {scale: [2f,2f,2f]}}
execute as @e[tag=test_file-e1] if score $test_file timer matches 30 run data merge entity @s {start_interpolation:0,interpolation_duration:30,transformation: {scale: [2f,2f,2f]}}

execute if score $test_file timer matches 0 run scoreboard players set $test_file timer -1
execute if score $test_file timer matches 0 run scoreboard players set $test_file flags 0
                
scoreboard players add $test_file timer 1