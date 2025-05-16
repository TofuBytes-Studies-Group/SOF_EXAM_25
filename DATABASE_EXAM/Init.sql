CREATE TABLE IF NOT EXISTS weapon (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    weapon_type UUID,
    damage INT
);

CREATE TABLE IF NOT EXISTS inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gold INT NOT NULL
);

CREATE TABLE IF NOT EXISTS weapon_inventory (
    inventory_id UUID,
    weapon_id UUID,
    FOREIGN KEY (inventory_id) REFERENCES inventory(id),
    FOREIGN KEY (weapon_id) REFERENCES weapon(id)
);

CREATE TABLE IF NOT EXISTS player (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    health INT NOT NULL,
    inventory_id UUID,
    FOREIGN KEY (inventory_id) REFERENCES inventory(id)
);

CREATE TABLE IF NOT EXISTS monster (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    health INT NOT NULL
);

CREATE TABLE IF NOT EXISTS world (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lore_id UUID NOT NULL,
    player_id UUID,
    FOREIGN KEY (player_id) REFERENCES player(id)
);

CREATE TABLE IF NOT EXISTS world_monster (
    world_id UUID,
    monster_id UUID,
    FOREIGN KEY (world_id) REFERENCES world(id),
    FOREIGN KEY (monster_id) REFERENCES monster(id)
);
