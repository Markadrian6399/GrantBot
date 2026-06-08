CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE repos (
  id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  owner            TEXT NOT NULL,
  repo_name        TEXT NOT NULL,
  webhook_secret   TEXT NOT NULL,
  payout_amount    FLOAT8 NOT NULL,
  daily_cap        FLOAT8 NOT NULL,
  owner_address    TEXT NOT NULL,
  delegation_hex   TEXT,
  created_at       TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE contributors (
  id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  github_username  TEXT NOT NULL,
  wallet_address   TEXT NOT NULL,
  repo_id          UUID NOT NULL REFERENCES repos(id) ON DELETE CASCADE
);

CREATE TABLE pr_events (
  id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  pr_number        INT NOT NULL,
  pr_title         TEXT NOT NULL,
  contributor      TEXT NOT NULL,
  repo_id          UUID NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
  merged_at        TIMESTAMPTZ NOT NULL,
  status           TEXT NOT NULL DEFAULT 'pending',
  created_at       TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE payments (
  id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  pr_event_id      UUID NOT NULL UNIQUE REFERENCES pr_events(id),
  amount           FLOAT8 NOT NULL,
  tx_hash          TEXT,
  venice_reason    TEXT NOT NULL,
  created_at       TIMESTAMPTZ DEFAULT NOW()
);
