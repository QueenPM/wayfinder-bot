-- Your SQL goes here
create table accessory_levels (
    id serial primary key,
    accessory_id int not null references accessories(id),
    max_health int,
    resillience int,
    weapon_power int,
    ability_power int,
    crit_rating int,
    crit_power int,
    break_power int,
    phys_defense int,
    mag_defense int
)