CREATE OR REPLACE FUNCTION update_timestamp()
    RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE extension if not exists "uuid-ossp";
CREATE table if not exists chat_messages
(
    id                   uuid primary key                     default uuid_generate_v4(),
    user_id              bigint               not null,
    text                 text                        not null,
    command              VARCHAR(100)               not null,
    created_at           TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT current_timestamp,
    updated_at           TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT current_timestamp
);

CREATE OR REPLACE TRIGGER set_timestamp
    BEFORE UPDATE
    ON chat_messages
    FOR EACH ROW
EXECUTE FUNCTION update_timestamp();
