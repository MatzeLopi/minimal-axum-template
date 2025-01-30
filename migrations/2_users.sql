create table "users"
(
    id       uuid primary key,
    username      text collate "case_insensitive" unique not null,
    email         text collate "case_insensitive" unique not null,
    is_verified   boolean                                not null default false,
    password_hash text                                   not null,
    verification_token text,
    created_at    timestamptz                            not null default now(),
    updated_at    timestamptz
);

SELECT trigger_updated_at('"users"');