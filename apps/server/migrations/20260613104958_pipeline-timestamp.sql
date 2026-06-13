alter table pipeline add column created_at timestamptz not null default now();
alter table pipeline add column updated_at timestamptz not null default now();
