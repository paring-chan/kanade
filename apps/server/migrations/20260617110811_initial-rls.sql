ALTER TABLE user_team ENABLE ROW LEVEL SECURITY;

CREATE POLICY user_team_owned ON user_team
    FOR ALL
    TO public
    USING (
        user_id = NULLIF(current_setting('app.auth_user_id', true), '')::uuid
    );

CREATE OR REPLACE FUNCTION get_my_team_ids()
RETURNS uuid[]
LANGUAGE sql
STABLE SECURITY DEFINER
AS $$
    SELECT coalesce(array_agg(team_id), '{}'::uuid[])
    FROM user_team
    WHERE user_id = NULLIF(current_setting('app.auth_user_id', true), '')::uuid;
$$;

ALTER TABLE team ENABLE ROW LEVEL SECURITY;

CREATE POLICY team_accessible ON team FOR ALL
    USING (
        id = ANY(get_my_team_ids())
    );

ALTER TABLE repo ENABLE ROW LEVEL SECURITY;

CREATE POLICY repo_accessible ON repo
    FOR ALL
    USING (team_id = ANY(get_my_team_ids()));

ALTER TABLE pipeline ENABLE ROW LEVEL SECURITY;

CREATE POLICY pipeline_accessible ON pipeline
    FOR ALL
    USING (
        repo_id IN (SELECT id FROM repo)
    );

ALTER TABLE pipeline_job ENABLE ROW LEVEL SECURITY;

CREATE POLICY pipeline_job_accessible ON pipeline_job
    FOR ALL
    USING (
        pipeline_id IN (SELECT id FROM pipeline)
    );

ALTER TABLE pipeline_job_step ENABLE ROW LEVEL SECURITY;

CREATE POLICY pipeline_job_step_accessible ON pipeline_job_step
    FOR ALL
    USING (
        job_id IN (SELECT id FROM pipeline_job)
    );
