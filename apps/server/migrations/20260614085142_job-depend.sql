ALTER TABLE pipeline_job DROP COLUMN parent_id;

CREATE TABLE pipeline_job_depend(
    upstream_id UUID NOT NULL REFERENCES pipeline_job (id),
    downstream_id UUID NOT NULL REFERENCES pipeline_job (id),

    PRIMARY KEY (upstream_id, downstream_id)
);
