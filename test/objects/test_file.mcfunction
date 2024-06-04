# File generated using DiSPA
execute as @e[tag=test_obj,tag=test] if score $test_obj-test_anim timer matches 0 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation:{translation: [0f,1f,0f]}}
execute as @e[tag=test_obj,tag=test] if score $test_obj-test_anim timer matches 0 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation:{left_rotation: [0f,0.70710677f,0f,0.70710677f]}}
execute as @e[tag=test_obj,tag=test] if score $test_obj-test_anim timer matches 0 run data merge entity @s {start_interpolation:0,interpolation_duration:20,transformation:{scale: [2f,2f,2f]}}
execute as @e[tag=test_obj,tag=test] at @s if score $test_obj-test_anim timer matches 40 run summon block_display ~ ~ ~ {Tags:["test_obj","test_block"]}
execute as @e[tag=test_obj,tag=test_block] if score $test_obj-test_anim timer matches 40 run data merge entity @s {block_state:{Name:"id",Properties:{type:"top",waterlogged:"false"}}}
execute as @e[tag=test_obj,tag=test] at @s if score $test_obj-test_anim timer matches 40 run summon item_display ~ ~ ~ {Tags:["test_obj","test_item"]}
execute as @e[tag=test_obj,tag=test_item] if score $test_obj-test_anim timer matches 40 run item replace entity @s contents with netherite_sword[minecraft:enchantment_glint_override=1]
execute as @e[tag=test_obj,tag=test] at @s if score $test_obj-test_anim timer matches 40 run summon text_display ~ ~ ~ {Tags:["test_obj","test_text"]}
execute as @e[tag=test_obj,tag=test_text] if score $test_obj-test_anim timer matches 40 run data merge entity @s {text:'"This is some text"'}
execute as @e[tag=test_obj,tag=test_text] at @s if score $test_obj-test_anim timer matches 40 run tp @s ~0 ~1 ~0
execute if score $test_obj-test_anim timer matches 40 run setblock 100 100 100 stone
setblock 100 101 100 diamond_block

execute if score $test_obj-test_anim timer matches 40.. run scoreboard players set $test_obj-test_anim flags 0
execute if score $test_obj-test_anim timer matches 40.. run scoreboard players set $test_obj-test_anim timer -1

scoreboard players add $test_obj-test_anim timer 1