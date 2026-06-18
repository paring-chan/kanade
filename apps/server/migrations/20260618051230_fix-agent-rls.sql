DROP POLICY agent_modify_admin ON agent;

CREATE POLICY agent_modify_admin ON agent
    FOR ALL
    USING (is_global AND is_session_admin());
