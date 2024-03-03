# File generated using DiSPA
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 0 run data merge entity @s {start_interpolation:0,interpolation_duration:0,transformation: {translation: [1f,0f,0f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 10 run data merge entity @s {start_interpolation:0,interpolation_duration:0,transformation: {translation: [0f,1f,0f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 20 run data merge entity @s {start_interpolation:0,interpolation_duration:10,transformation: {translation: [0f,0f,1f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 30 run data merge entity @s {start_interpolation:0,interpolation_duration:0,transformation: {left_rotation: [0.70710677f,0f,0f,0.70710677f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 40 run data merge entity @s {start_interpolation:0,interpolation_duration:0,transformation: {left_rotation: [0.70710677f,0f,0.70710677f,0f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 50 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation: {left_rotation: [0.70710677f,0.70710677f,0f,0f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 60 run data merge entity @s {start_interpolation:0,interpolation_duration:30,transformation: {scale: [2f,1f,1f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 70 run data merge entity @s {start_interpolation:0,interpolation_duration:30,transformation: {scale: [1f,2f,1f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 80 run data merge entity @s {start_interpolation:0,interpolation_duration:40,transformation: {scale: [1f,2f,1f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 90 run data merge entity @s {start_interpolation:0,interpolation_duration:30,transformation: {translation: [1f,0f,1f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 90 run data merge entity @s {start_interpolation:0,interpolation_duration:30,transformation: {left_rotation: [0.49999997f,0.49999997f,0.49999997f,0.49999997f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 90 run data merge entity @s {start_interpolation:0,interpolation_duration:30,transformation: {scale: [2f,2f,1f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 60 run data merge entity @s {start_interpolation:0,interpolation_duration:50,transformation: {translation: [0f,1f,0f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 60 run data merge entity @s {start_interpolation:0,interpolation_duration:50,transformation: {left_rotation: [0.70710677f,0f,0.70710677f,0f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 60 run data merge entity @s {start_interpolation:0,interpolation_duration:50,transformation: {scale: [1f,3f,1f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 100 run data merge entity @s {start_interpolation:0,interpolation_duration:60,transformation: {translation: [0f,1f,1f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 100 run data merge entity @s {start_interpolation:0,interpolation_duration:60,transformation: {left_rotation: [0.49999997f,0.49999997f,0.49999997f,-0.49999997f]}}
execute as @e[tag=dtest-e1] if score $dtest-atest timer matches 100 run data merge entity @s {start_interpolation:0,interpolation_duration:60,transformation: {scale: [1f,3f,2f]}}

execute if score $dtest-atest timer matches 110.. run scoreboard players set $dtest-atest flags 0
execute if score $dtest-atest timer matches 110.. run scoreboard players set $dtest-atest timer -1
scoreboard players add $dtest-atest timer 1