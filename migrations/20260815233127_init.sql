-- Add migration script here
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS discord_to_artist (
    discord_id INTEGER PRIMARY KEY NOT NULL,
    artist_ascii_name TEXT NOT NULL UNIQUE
) STRICT;

CREATE INDEX idx_discord_to_artist_artist_ascii_name ON discord_to_artist (artist_ascii_name);

CREATE TABLE IF NOT EXISTS tickets(
    ticket_channel_id INTEGER UNIQUE NOT NULL,
    created_by INTEGER NOT NULL,
    created_on_tsec INTEGER NOT NULL,

    start_commit BLOB,
    path TEXT,
    status TEXT NOT NULL,


    FOREIGN KEY (created_by) REFERENCES discord_to_artist(discord_id)
        ON UPDATE CASCADE
        ON DELETE SET DEFAULT
) STRICT;
