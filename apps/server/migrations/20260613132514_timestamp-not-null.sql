alter table team alter column created_at set not null;
alter table team alter column updated_at set not null;

alter table user_team alter column created_at set not null;
alter table user_team alter column updated_at set not null;

alter table secret alter column created_at set not null;
alter table secret alter column updated_at set not null;

alter table team_secret alter column created_at set not null;
alter table team_secret alter column updated_at set not null;

alter table repo_secret alter column created_at set not null;
alter table repo_secret alter column updated_at set not null;
