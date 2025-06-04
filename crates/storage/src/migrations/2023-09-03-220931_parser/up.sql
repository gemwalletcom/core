CREATE TABLE parser_state (
    chain VARCHAR NOT NULL PRIMARY KEY REFERENCES chains (id) ON DELETE CASCADE,
    current_block INTEGER NOT NULL default 0,
    latest_block INTEGER NOT NULL default 0,
    await_blocks INTEGER NOT NULL default 0,
    timeout_between_blocks INTEGER NOT NULL default 0,
    timeout_latest_block INTEGER NOT NULL default 0,
    parallel_blocks INTEGER NOT NULL default 1,
    is_enabled boolean NOT NULL default true,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    queue_after_blocks INTEGER,
    UNIQUE(chain)
);

SELECT diesel_manage_updated_at('parser_state');