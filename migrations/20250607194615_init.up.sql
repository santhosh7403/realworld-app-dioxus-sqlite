
CREATE TABLE IF NOT EXISTS Users(
    username text NOT NULL PRIMARY KEY,
    email text NOT NULL UNIQUE,
    password text NOT NULL,
    bio text NULL,
    image text NULL
);

CREATE TABLE IF NOT EXISTS Follows(
    follower text NOT NULL REFERENCES Users(username) ON DELETE CASCADE ON UPDATE CASCADE,
    influencer text NOT NULL REFERENCES Users(username) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY (follower, influencer)
);

CREATE TABLE IF NOT EXISTS Articles(
    slug text NOT NULL PRIMARY KEY,
    author text NOT NULL REFERENCES Users(username) ON DELETE CASCADE ON UPDATE CASCADE,
    title text NOT NULL,
    description text NOT NULL,
    body text NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ArticleTags(
    article text NOT NULL REFERENCES Articles(slug) ON DELETE CASCADE ON UPDATE CASCADE,
    tag text NOT NULL,
    PRIMARY KEY (article, tag)
);

CREATE INDEX IF NOT EXISTS tags ON ArticleTags(tag);

CREATE TABLE IF NOT EXISTS FavArticles(
    article text NOT NULL REFERENCES Articles(slug) ON DELETE CASCADE ON UPDATE CASCADE,
    username text NOT NULL REFERENCES Users(username) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY (article, username)
);

CREATE TABLE IF NOT EXISTS Comments(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    article text NOT NULL REFERENCES Articles(slug) ON DELETE CASCADE ON UPDATE CASCADE,
    username text NOT NULL REFERENCES Users(username) ON DELETE CASCADE ON UPDATE CASCADE,
    body text NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE VIRTUAL TABLE IF NOT EXISTS Articles_fts using fts5(
    slug UNINDEXED,
    title,
    description,
    body,
    content='Articles',
    tokenize='porter'
);

CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
    INSERT INTO articles_fts(rowid, slug, title, description, body) VALUES (NEW.oid, NEW.slug, NEW.title,NEW.description, NEW.body);
END;

-- Update trigger: When a document is updated, delete old entry and insert new one into FTS5
CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, slug, title, description, body) VALUES ('delete', OLD.oid, OLD.slug, OLD.title, OLD.description, OLD.body);
    INSERT INTO articles_fts(rowid, slug, title, description, body) VALUES (NEW.oid, NEW.slug, NEW.title, NEW.description, NEW.body);
END;

-- Delete trigger: When a document is deleted, remove it from FTS5
CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, slug, title, description, body) VALUES ('delete', OLD.oid, OLD.slug, OLD.title, OLD.description, OLD.body);
END;

CREATE VIRTUAL TABLE IF NOT EXISTS articletags_fts using fts5(
    article UNINDEXED,
    tag,
    content='articletags',
    tokenize='porter'
);

CREATE TRIGGER IF NOT EXISTS articletags_ai AFTER INSERT ON articletags BEGIN
    INSERT INTO articletags_fts(rowid, article, tag) VALUES (NEW.oid, NEW.article, NEW.tag);
END;

-- Update trigger: When a document is updated, delete old entry and insert new one into FTS5
CREATE TRIGGER IF NOT EXISTS articletags_au AFTER UPDATE ON articletags BEGIN
    INSERT INTO articletags_fts(articletags_fts, rowid, article, tag) VALUES ('delete', OLD.oid, OLD.article, OLD.tag);
    INSERT INTO articletags_fts(rowid, article, tag) VALUES (NEW.oid, NEW.article, NEW.tag);
END;

-- Delete trigger: When a document is deleted, remove it from FTS5
CREATE TRIGGER IF NOT EXISTS articletags_ad AFTER DELETE ON articletags BEGIN
    INSERT INTO articletags_fts(articletags_fts, rowid, article, tag) VALUES ('delete', OLD.oid, OLD.article, OLD.tag);
END;




-- sqlite> select snippet(articles_fts, 2, '==========', '===========', '....',32)   from articles_fts as af join articles a on af.rowid=a.oid where af.body match 'gene' ;
-- sqlite> select a.slug, highlight(articles_fts, 2, '*****', '*****')   from articles_fts as af join articles a on af.rowid=a.oid where af.body match 'develop' ;


-- sqlite> select snippet(articles_fts,0, '***','****','>>>',2), snippet(articles_fts, 1, '****', '****' , '>>>' , 3) ,snippet(articles_fts, 2, '==========', '===========', '....',2)   from articles_fts as af join articles a on af.rowid=a.oid where articles_fts match 'colonize' limit 4 ;
-- Mars ***Colonization****>>>|Examining the challenges>>>|....==========colonizing=========== Mars....
-- sqlite> select snippet(articles_fts,0, '***','****','>>>',2), snippet(articles_fts, 1, '****', '****' , '>>>' , 3) ,snippet(articles_fts, 2, '==========', '===========', '....',2)   from articles_fts as af join articles a on af.rowid=a.oid where articles_fts match 'colonize' limit 4 ;
