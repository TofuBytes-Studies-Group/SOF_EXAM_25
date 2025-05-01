CREATE TABLE IF NOT EXISTS weapon (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    weapon_type CHAR(36),
    damage INT
);


CREATE TABLE IF NOT EXISTS inventory (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    gold INT NOT NULL
);


CREATE TABLE IF NOT EXISTS weapon_inventory (
    inventory_id CHAR(36),
    weapon_id CHAR(36),
    FOREIGN KEY (inventory_id) REFERENCES inventory(id),
    FOREIGN KEY (weapon_id) REFERENCES weapon(id)
); 


CREATE TABLE IF NOT EXISTS player (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    name VARCHAR(100) NOT NULL,
    health INT NOT NULL,
    inventory_id CHAR(36),
    FOREIGN KEY (inventory_id) REFERENCES inventory(id)
);


CREATE TABLE IF NOT EXISTS monster (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    name VARCHAR(100) NOT NULL,
    health INT NOT NULL
);

CREATE TABLE IF NOT EXISTS world (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    lore_id char(36) not null,
    player_id Char(36),
    FOREIGN KEY (player_id) REFERENCES player(id)
);

CREATE TABLE IF NOT EXISTS world_monster (
    world_id CHAR(36),
    monster_id CHAR(36),
    FOREIGN KEY (world_id) REFERENCES world(id),
    FOREIGN KEY (monster_id) REFERENCES monster(id)
);


