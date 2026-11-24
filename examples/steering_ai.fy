// Smooth steering AI combining `dir_to_player`, `uwu`, and velocity clamping
let pos = get_self_pos();
let target = get_player_pos();
let desired = dir_to_player() * 220.0;
let current = get_self_vel();
let blended = current uwu desired;
set_self_vel(blended);
purr("Steering toward player at dist", distance2(pos, target));
return blended;
