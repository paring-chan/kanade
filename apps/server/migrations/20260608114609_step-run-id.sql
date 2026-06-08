TRUNCATE pipeline_job_step_run;
ALTER TABLE pipeline_job_step_run
    ADD COLUMN id uuid NOT NULL PRIMARY KEY;
