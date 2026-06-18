CREATE OR REPLACE FUNCTION is_rls_bypassed()
RETURNS boolean
LANGUAGE sql
STABLE SECURITY DEFINER
AS $$
    SELECT current_setting('app.bypass_rls', true) = 'true';
$$;

CREATE OR REPLACE FUNCTION is_session_admin()
RETURNS boolean
LANGUAGE sql
STABLE SECURITY DEFINER
AS $$
    SELECT coalesce(
        (SELECT is_admin FROM "user" WHERE id = NULLIF(current_setting('app.auth_user_id', true), '')::uuid),
        false
    );
$$;

CREATE POLICY agent_modify_bypass ON agent
    FOR ALL
    USING (is_rls_bypassed());

CREATE POLICY agent_modify_admin ON agent
    FOR ALL
    USING (is_session_admin());
