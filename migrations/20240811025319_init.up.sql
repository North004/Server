-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
        id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
        username VARCHAR(100) NOT NULL UNIQUE,
        email VARCHAR(255) NOT NULL UNIQUE,
        password VARCHAR(100) NOT NULL,
        role VARCHAR(50) NOT NULL DEFAULT 'user',
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE INDEX user_username_idx ON users (username);

CREATE TABLE profiles (
        id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
        user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
        photo VARCHAR NOT NULL DEFAULT 'default.png',
        bio TEXT,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);


CREATE TABLE posts (
        id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
        author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        title VARCHAR(50) NOT NULL,
        content VARCHAR(400) NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE INDEX posts_title_idx ON posts(title);

CREATE TABLE post_reactions (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_like BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
