# File generated using DiSPA
execute as @e[tag=dtest-e1] if score $dtest-dtest timer matches 0 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation: {translation: [0f,1f,0f]}}
execute as @e[tag=dtest-e1] if score $dtest-dtest timer matches 10 run data merge entity @s {start_interpolation:0,interpolation_duration:21,transformation: {translation: [0f,2f,0f]}}
execute as @e[tag=dtest-e1] if score $dtest-dtest timer matches 10 run data merge entity @s {start_interpolation:0,interpolation_duration:22,transformation: {left_rotation: [0.70710677f,0f,0f,0.70710677f]}}
execute as @e[tag=dtest-e1] if score $dtest-dtest timer matches 20 run data merge entity @s {start_interpolation:0,interpolation_duration:23,transformation: {translation: [1f,0f,0f]}}
execute as @e[tag=dtest-e1] if score $dtest-dtest timer matches 20 run data merge entity @s {start_interpolation:0,interpolation_duration:24,transformation: {left_rotation: [0.70710677f,0f,0.70710677f,0f]}}
execute as @e[tag=dtest-e1] if score $dtest-dtest timer matches 30 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation: {translation: [2f,0f,0f]}}
execute as @e[tag=dtest-e1] if score $dtest-dtest timer matches 30 run data merge entity @s {start_interpolation:0,interpolation_duration:10,transformation: {translation: [2f,0f,1f]}}
execute as @e[tag=dtest-e1] if score $dtest-dtest timer matches 30 run data merge entity @s {start_interpolation:0,interpolation_duration:10,transformation: {left_rotation: [0.69463193f,0.006061961f,0.006277344f,0.71931237f]}}
execute as @e[tag=dtest-e1] if score $dtest-dtest timer matches 30 run data merge entity @s {start_interpolation:0,interpolation_duration:100,transformation: {scale: [2f,2f,2f]}}
execute as @e[tag=dtest-e1] if score $dtest-dtest timer matches 30 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation: {scale: [2f,2f,2f]}}

execute if score $dtest-dtest timer matches 100.. run scoreboard players set $dtest-dtest flags 0
execute if score $dtest-dtest timer matches 100.. run scoreboard players set $dtest-dtest timer -1

scoreboard players add $dtest-dtest timer 1