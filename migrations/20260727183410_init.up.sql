CREATE TYPE task_status_enum AS ENUM ('start', 'todo', 'done', 'cancelled');
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
    user_id            UUID        DEFAULT gen_random_uuid() PRIMARY KEY,
    name               varchar(255),
    email              varchar(255)              NOT NULL UNIQUE,
    password           varchar(255)              NOT NULL,
    email_is_confirmed boolean     DEFAULT false NOT NULL,
    created_at         timestamptz DEFAULT now() NOT NULL,
    updated_at         timestamptz DEFAULT now() NOT NULL
);
CREATE TABLE teams
(
    team_id    UUID        DEFAULT gen_random_uuid() PRIMARY KEY,
    name       varchar(255)              NOT NULL UNIQUE,                                            -- название команды уникально
    created_by UUID                      NOT NULL,                                                   -- кто создал команду
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT fk_teams_users FOREIGN KEY (created_by) REFERENCES users (user_id) ON DELETE RESTRICT -- нельзя удалить команду после удаления пользователя
);
CREATE TABLE team_members
(
    team_id    UUID                      NOT NULL,
    user_id    UUID                      NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL, -- когда user присоединился к команде
    CONSTRAINT pk_team_members PRIMARY KEY (team_id, user_id),
    CONSTRAINT fk_team_members_teams FOREIGN KEY (team_id) REFERENCES teams (team_id) ON DELETE CASCADE,
    CONSTRAINT fk_team_members_users FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
CREATE TABLE tasks
(
    task_id     UUID             DEFAULT gen_random_uuid() PRIMARY KEY,
    name        varchar(255)                     NOT NULL, -- названия могут повторяться
    description TEXT,
    created_by  UUID                             NOT NULL, -- кто создал задачу, берем из таблицы users
    team_id     UUID                             NOT NULL, -- задача для команды
    assignee_id UUID,                                      -- кто из user-ов принял в работу задачу
    status      task_status_enum DEFAULT 'start' NOT NULL,
    created_at  timestamptz      DEFAULT now()   NOT NULL,
    updated_at  timestamptz      DEFAULT now()   NOT NULL,
    CONSTRAINT fk_tasks_teams FOREIGN KEY (team_id) REFERENCES teams (team_id) ON DELETE RESTRICT,
    CONSTRAINT fk_tasks_users1 FOREIGN KEY (created_by) REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_tasks_users2 FOREIGN KEY (assignee_id) REFERENCES users (user_id) ON DELETE SET NULL
);
CREATE TABLE task_histories
(
    task_history_id UUID        DEFAULT gen_random_uuid() PRIMARY KEY,
    task_id         UUID                      NOT NULL,
    user_id         UUID                      NOT NULL,
    msg             TEXT                      NOT NULL,
    created_at      timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT fk_task_histories_tasks FOREIGN KEY (task_id) REFERENCES tasks (task_id) ON DELETE CASCADE,
    CONSTRAINT fk_task_histories_users FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
CREATE TABLE task_comments
(
    task_comment_id UUID        DEFAULT gen_random_uuid() PRIMARY KEY,
    task_id         UUID                      NOT NULL,
    user_id         UUID                      NOT NULL,
    msg             TEXT                      NOT NULL,
    created_at      timestamptz DEFAULT now() NOT NULL,
    updated_at      timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT fk_task_histories_tasks FOREIGN KEY (task_id) REFERENCES tasks (task_id) ON DELETE CASCADE,
    CONSTRAINT fk_task_histories_users FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);

CREATE TRIGGER trg_users_updated
    BEFORE UPDATE
    ON users
    FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER trg_teams_updated
    BEFORE UPDATE
    ON teams
    FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER trg_tasks_updated
    BEFORE UPDATE
    ON tasks
    FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER trg_task_comments_updated
    BEFORE UPDATE
    ON task_comments
    FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();