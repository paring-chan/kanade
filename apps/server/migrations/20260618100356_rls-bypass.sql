CREATE POLICY user_team_bypass ON user_team FOR ALL USING (is_rls_bypassed());
CREATE POLICY team_bypass ON team FOR ALL USING (is_rls_bypassed());
CREATE POLICY repo_bypasss ON repo FOR ALL USING (is_rls_bypassed());
CREATE POLICY pipeline_bypass ON pipeline FOR ALL USING (is_rls_bypassed());
CREATE POLICY pipeline_job_bypass ON pipeline_job FOR ALL USING (is_rls_bypassed());
CREATE POLICY pipeline_job_step_bypass ON pipeline_job_step FOR ALL USING (is_rls_bypassed());
