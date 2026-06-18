CREATE POLICY agent_pipeline_job_access ON pipeline_job
    FOR ALL
    USING (agent_id = NULLIF(current_setting('app.auth_agent_id', true), '')::uuid);
