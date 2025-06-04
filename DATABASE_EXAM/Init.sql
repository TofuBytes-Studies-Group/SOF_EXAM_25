-- Tables

CREATE TABLE IF NOT EXISTS weapon (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    damage INT NOT NULL,
    weight FLOAT NOT NULL,
    upgrade VARCHAR(100) NOT NULL,
    perk VARCHAR(100) NOT NULL,
    weapon_type VARCHAR(50) NOT NULL,
    predicted_price FLOAT NOT NULL
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
    hp INT NOT NULL,
    max_hp INT NOT NULL,
    defense INT NOT NULL,
    strength INT NOT NULL,
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

-- Functions

CREATE OR REPLACE FUNCTION get_player_stats(p_name TEXT)
RETURNS TABLE (
    player_name VARCHAR(100),
    player_hp INT,
    player_max_hp INT,
    player_defense INT,
    player_strength INT,
    inventory_gold INT,
    weapon_name VARCHAR(100),
    weapon_damage INT,
    weapon_weight FLOAT,
    weapon_upgrade VARCHAR(100),
    weapon_perk VARCHAR(100),
    weapon_type VARCHAR(50),
    weapon_predicted_price FLOAT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        p.name,
        p.hp,
        p.max_hp,
        p.defense,
        p.strength,
        i.gold,
        w.name,
        w.damage,
        w.weight,
        w.upgrade,
        w.perk,
        w.weapon_type,
        w.predicted_price
    FROM player p
    JOIN inventory i ON p.inventory_id = i.id
    LEFT JOIN weapon_inventory wi ON i.id = wi.inventory_id
    LEFT JOIN weapon w ON wi.weapon_id = w.id
    WHERE p.name = p_name;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION create_inventory_for_new_player()
RETURNS TRIGGER AS $$
DECLARE
    new_inventory_id UUID;
BEGIN

    INSERT INTO inventory (gold) VALUES (1000) RETURNING id INTO new_inventory_id;

    UPDATE player SET inventory_id = new_inventory_id WHERE id = NEW.id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers

CREATE TRIGGER trg_create_inventory_after_player_insert
AFTER INSERT ON player
FOR EACH ROW
EXECUTE FUNCTION create_inventory_for_new_player();
