CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_updated_at BEFORE UPDATE ON "user" FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
CREATE TRIGGER forge_updated_at BEFORE UPDATE ON "forge" FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
CREATE TRIGGER user_forge_updated_at BEFORE UPDATE ON "user_forge" FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
CREATE TRIGGER team_updated_at BEFORE UPDATE ON "team" FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
CREATE TRIGGER user_team_updated_at BEFORE UPDATE ON "user_team" FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
CREATE TRIGGER secret_updated_at BEFORE UPDATE ON "secret" FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
CREATE TRIGGER team_secret_updated_at BEFORE UPDATE ON "team_secret" FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
CREATE TRIGGER repo_updated_at BEFORE UPDATE ON "repo" FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
CREATE TRIGGER repo_secret_updated_at BEFORE UPDATE ON "repo_secret" FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
CREATE TRIGGER pipeline_updated_at BEFORE UPDATE ON "pipeline" FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
CREATE TRIGGER agent_updated_at BEFORE UPDATE ON "agent" FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
