-- Table des utilisateurs (obligatoire)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    phone TEXT,
    country TEXT,
    wallet_address TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Table des idées brutes
CREATE TABLE IF NOT EXISTS ideas (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    raw_idea TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Table des résumés IA
CREATE TABLE IF NOT EXISTS summaries (
    id UUID PRIMARY KEY,
    idea_id UUID NOT NULL REFERENCES ideas(id),
    title TEXT NOT NULL,
    problem TEXT NOT NULL,
    solution TEXT NOT NULL,
    claim TEXT NOT NULL, -- Une seule revendication pour MVP
    cpc_code TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Table des preuves Hedera
CREATE TABLE IF NOT EXISTS proofs (
    id UUID PRIMARY KEY,
    summary_id UUID NOT NULL REFERENCES summaries(id),
    hash TEXT NOT NULL,
    hedera_tx_id TEXT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Index pour performances
CREATE INDEX IF NOT EXISTS idx_ideas_user_id ON ideas(user_id);
CREATE INDEX IF NOT EXISTS idx_summaries_idea_id ON summaries(idea_id);
CREATE INDEX IF NOT EXISTS idx_proofs_summary_id ON proofs(summary_id);