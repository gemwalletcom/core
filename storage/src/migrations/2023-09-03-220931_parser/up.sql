CREATE TABLE parser_state (
    chain VARCHAR NOT NULL PRIMARY KEY,
    current_block INTEGER NOT NULL default 0,
    latest_block INTEGER NOT NULL default 0,
    await_blocks INTEGER NOT NULL default 0,
    is_enabled boolean NOT NULL default true,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(chain)
);

SELECT diesel_manage_updated_at('parser_state');

/*

INSERT INTO "parser_state" ("chain", "current_block", "latest_block") VALUES ('ethereum', 18058858, 18058958);

*/