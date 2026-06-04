CREATE TYPE "role_type" AS ENUM (
  'viewer',
  'manager',
  'admin'
);

CREATE TYPE "event_type" AS ENUM (
  'push',
  'tag',
  'release',
  'pull_request',
  'cron',
  'manual'
);

CREATE TYPE "pipeline_status" AS ENUM (
  'evaluating',
  'queued',
  'running',
  'success',
  'failed',
  'cancelled'
);

CREATE TYPE "job_status" AS ENUM (
  'waiting',
  'pending',
  'running',
  'success',
  'failed',
  'skipped',
  'cancelled'
);

CREATE TYPE "agent_status" AS ENUM (
  'idle',
  'busy',
  'offline'
);

CREATE TABLE "user" (
  "id" uuid PRIMARY KEY NOT NULL,
  "username" text UNIQUE NOT NULL,
  "nick" text,
  "email" text UNIQUE,
  "created_at" timestamptz DEFAULT (now()),
  "updated_at" timestamptz DEFAULT (now())
);

CREATE TABLE "forge" (
  "id" uuid PRIMARY KEY NOT NULL,
  "name" text,
  "config" jsonb NOT NULL,
  "created_at" timestamptz DEFAULT (now()),
  "updated_at" timestamptz DEFAULT (now())
);

CREATE TABLE "user_forge" (
  "id" uuid PRIMARY KEY NOT NULL,
  "user_id" uuid NOT NULL,
  "forge_id" uuid NOT NULL,
  "forge_user_id" text NOT NULL,
  "access_token" text NOT NULL,
  "refresh_token" text NOT NULL,
  "access_token_expires_at" timestamptz NOT NULL,
  "created_at" timestamptz DEFAULT (now()),
  "updated_at" timestamptz DEFAULT (now())
);

CREATE TABLE "team" (
  "id" uuid PRIMARY KEY NOT NULL,
  "name" text,
  "slug" text UNIQUE NOT NULL,
  "created_at" timestamptz DEFAULT (now()),
  "updated_at" timestamptz DEFAULT (now())
);

CREATE TABLE "user_team" (
  "id" uuid PRIMARY KEY NOT NULL,
  "user_id" uuid NOT NULL,
  "team_id" uuid NOT NULL,
  "role" role_type NOT NULL,
  "created_at" timestamptz DEFAULT (now()),
  "updated_at" timestamptz DEFAULT (now())
);

CREATE TABLE "secret" (
  "id" uuid PRIMARY KEY,
  "key" text,
  "value" bytea,
  "created_at" timestamptz DEFAULT (now()),
  "updated_at" timestamptz DEFAULT (now())
);

CREATE TABLE "team_secret" (
  "id" uuid PRIMARY KEY NOT NULL,
  "team_id" uuid NOT NULL,
  "secret_id" uuid NOT NULL,
  "scopes" event_type[],
  "created_at" timestamptz DEFAULT (now()),
  "updated_at" timestamptz DEFAULT (now())
);

CREATE TABLE "repo" (
  "id" uuid PRIMARY KEY NOT NULL,
  "name" text NOT NULL,
  "slug" text NOT NULL,
  "team_id" uuid NOT NULL,
  "forge_id" uuid NOT NULL,
  "forge_repo_id" text NOT NULL,
  "forge_webhook_token" bytea NOT NULL
);

CREATE TABLE "repo_secret" (
  "id" uuid PRIMARY KEY NOT NULL,
  "repo_id" uuid NOT NULL,
  "secret_id" uuid NOT NULL,
  "scopes" event_type[],
  "created_at" timestamptz DEFAULT (now()),
  "updated_at" timestamptz DEFAULT (now())
);

CREATE TABLE "pipeline" (
  "id" uuid PRIMARY KEY NOT NULL,
  "serial" integer NOT NULL,
  "repo_id" uuid NOT NULL,
  "title" text,
  "triggered_by" text NOT NULL,
  "triggered_by_user" uuid,
  "event_type" event_type NOT NULL,
  "event_payload" jsonb NOT NULL DEFAULT '{}',
  "git_ref" text,
  "git_commit_id" text NOT NULL,
  "status" pipeline_status NOT NULL DEFAULT 'pending'
);

CREATE TABLE "pipeline_job" (
  "id" uuid PRIMARY KEY NOT NULL,
  "pipeline_id" uuid NOT NULL,
  "depends_on" uuid[],
  "key" text NOT NULL,
  "name" text NOT NULL,
  "image" text,
  "env" jsonb NOT NULL DEFAULT '{}',
  "timeout" int NOT NULL DEFAULT 60,
  "created_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE TABLE "pipeline_job_step" (
  "id" uuid PRIMARY KEY NOT NULL,
  "job_id" uuid NOT NULL,
  "name" text NOT NULL,
  "env" jsonb NOT NULL DEFAULT '{}',
  "ordering" int NOT NULL,
  "command" text NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE TABLE "pipeline_job_run" (
  "id" uuid PRIMARY KEY NOT NULL,
  "job_id" uuid NOT NULL,
  "attempt_serial" int NOT NULL,
  "env" jsonb NOT NULL DEFAULT '{}',
  "status" job_status NOT NULL DEFAULT 'pending',
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "started_at" timestamptz,
  "finished_at" timestamptz,
  "agent_id" uuid
);

CREATE TABLE "pipeline_job_step_run" (
  "run_id" uuid NOT NULL,
  "step_id" uuid NOT NULL,
  "status" job_status NOT NULL DEFAULT 'pending',
  "exit_code" int,
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "started_at" timestamptz,
  "finished_at" timestamptz
);

CREATE TABLE "agent" (
  "id" uuid PRIMARY KEY NOT NULL,
  "name" text NOT NULL,
  "status" agent_status,
  "token_sha256" bytea UNIQUE NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "updated_at" timestamptz NOT NULL DEFAULT (now()),
  "last_heartbeat_at" timestamptz
);

CREATE TABLE "agent_tag" (
  "id" uuid PRIMARY KEY NOT NULL,
  "agent_id" uuid NOT NULL,
  "tag" text NOT NULL
);

CREATE UNIQUE INDEX ON "user_forge" ("user_id", "forge_id");

CREATE UNIQUE INDEX ON "user_forge" ("forge_id", "forge_user_id");

CREATE UNIQUE INDEX ON "user_team" ("user_id", "team_id");

CREATE UNIQUE INDEX ON "team_secret" ("team_id", "secret_id");

CREATE UNIQUE INDEX ON "repo" ("forge_id", "forge_repo_id");

CREATE UNIQUE INDEX ON "repo" ("team_id", "slug");

CREATE UNIQUE INDEX ON "repo_secret" ("repo_id", "secret_id");

CREATE UNIQUE INDEX ON "pipeline" ("repo_id", "serial");

CREATE UNIQUE INDEX ON "pipeline_job" ("pipeline_id", "key");

CREATE UNIQUE INDEX ON "pipeline_job_run" ("job_id", "attempt_serial");

CREATE UNIQUE INDEX ON "pipeline_job_step_run" ("run_id", "step_id");

CREATE UNIQUE INDEX ON "agent_tag" ("agent_id", "tag");

ALTER TABLE "user_forge" ADD FOREIGN KEY ("user_id") REFERENCES "user" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "user_forge" ADD FOREIGN KEY ("forge_id") REFERENCES "forge" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "user_team" ADD FOREIGN KEY ("user_id") REFERENCES "user" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "user_team" ADD FOREIGN KEY ("team_id") REFERENCES "team" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "team_secret" ADD FOREIGN KEY ("team_id") REFERENCES "team" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "team_secret" ADD FOREIGN KEY ("secret_id") REFERENCES "secret" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "repo" ADD FOREIGN KEY ("team_id") REFERENCES "team" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "repo" ADD FOREIGN KEY ("forge_id") REFERENCES "forge" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "repo_secret" ADD FOREIGN KEY ("repo_id") REFERENCES "repo" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "repo_secret" ADD FOREIGN KEY ("secret_id") REFERENCES "secret" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "pipeline" ADD FOREIGN KEY ("repo_id") REFERENCES "repo" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "pipeline" ADD FOREIGN KEY ("triggered_by_user") REFERENCES "user" ("id") ON DELETE SET NULL DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "pipeline_job" ADD FOREIGN KEY ("pipeline_id") REFERENCES "pipeline" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "pipeline_job_step" ADD FOREIGN KEY ("job_id") REFERENCES "pipeline_job" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "pipeline_job_run" ADD FOREIGN KEY ("job_id") REFERENCES "pipeline_job" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "pipeline_job_run" ADD FOREIGN KEY ("agent_id") REFERENCES "agent" ("id") ON DELETE SET NULL DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "pipeline_job_step_run" ADD FOREIGN KEY ("run_id") REFERENCES "pipeline_job_run" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "pipeline_job_step_run" ADD FOREIGN KEY ("step_id") REFERENCES "pipeline_job_step" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "agent_tag" ADD FOREIGN KEY ("agent_id") REFERENCES "agent" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;
