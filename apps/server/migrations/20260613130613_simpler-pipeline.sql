drop table pipeline_job_step_run;
drop table pipeline_job_run;

alter table pipeline_job drop column depends_on;
alter table pipeline_job add column parent_id uuid references pipeline_job (id);
alter table pipeline_job add column started_at timestamptz;
alter table pipeline_job add column finished_at timestamptz;
alter table pipeline_job add column status job_status not null default 'pending'::job_status;
alter table pipeline_job add column agent_id uuid references agent (id);

alter table pipeline_job_step add column status job_status not null default 'pending'::job_status;
alter table pipeline_job_step add column exit_code int;
alter table pipeline_job_step add column started_at timestamptz;
alter table pipeline_job_step add column finished_at timestamptz;
