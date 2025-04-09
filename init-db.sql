-- Create users table
CREATE TABLE IF NOT EXISTS users (
                                     id SERIAL PRIMARY KEY,
                                     username TEXT NOT NULL
);

-- Insert some example data
INSERT INTO users (username) VALUES ('john_doe');
INSERT INTO users (username) VALUES ('jane_smith');