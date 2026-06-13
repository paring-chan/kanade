ALTER TABLE pipeline ALTER COLUMN status DROP DEFAULT;
ALTER TYPE pipeline_status RENAME TO pipeline_status_old;

CREATE TYPE pipeline_status AS ENUM (
  'queued',
  'running',
  'success',
  'failed',
  'cancelled'
);

ALTER TABLE pipeline ALTER COLUMN status TYPE pipeline_status USING status::text::pipeline_status;

DROP TYPE pipeline_status_old;
