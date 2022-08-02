-- -------------------------------------------------------------
-- TablePlus 4.8.2(436)
--
-- https://tableplus.com/
--
-- Database: postgres
-- Generation Time: 2022-08-02 14:22:39.5680
-- -------------------------------------------------------------


-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."results" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "ip" text NOT NULL,
    "motd" text,
    "max_players" int8 NOT NULL,
    "online_players" int8 NOT NULL,
    "version_name" text,
    "protocol_version" int8 NOT NULL,
    "player_sample" _uuid,
    "timestamp" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "favicon" text,
    PRIMARY KEY ("id")
);

