CREATE
    OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at
        = NOW();
    RETURN NEW;
END;
$$
    LANGUAGE plpgsql;
CREATE TABLE users
(
    user_id    UUID        DEFAULT gen_random_uuid() PRIMARY KEY,
    fio        varchar(255)              NOT NULL,
    email      varchar(255)              NOT NULL UNIQUE,
    password   varchar(255)              NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL
);
CREATE TABLE tasks
(
    task_id    UUID        DEFAULT gen_random_uuid() PRIMARY KEY,
    name       varchar(255)              NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL
);
CREATE TRIGGER trg_users_updated
    BEFORE UPDATE
    ON users
    FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER trg_tasks_updated
    BEFORE UPDATE
    ON tasks
    FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();