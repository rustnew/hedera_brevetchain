-- Création de la table users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    phone TEXT,
    country TEXT,
    wallet_address TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE patent_drafts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    raw_idea TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    hedera_tx_id TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE structured_patents (
    id UUID PRIMARY KEY,
    patent_draft_id UUID NOT NULL,
    title TEXT NOT NULL,
    problem TEXT NOT NULL,
    solution TEXT NOT NULL,
    claims JSONB NOT NULL,
    cpc_code TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (patent_draft_id) REFERENCES patent_drafts(id)
);

-- Index pour améliorer les performances
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_patent_drafts_user_id ON patent_drafts(user_id);
CREATE INDEX idx_patent_drafts_status ON patent_drafts(status);
CREATE INDEX idx_structured_patents_draft_id ON structured_patents(patent_draft_id);
CREATE INDEX idx_patent_drafts_created_at ON patent_drafts(created_at);
CREATE INDEX idx_structured_patents_created_at ON structured_patents(created_at);