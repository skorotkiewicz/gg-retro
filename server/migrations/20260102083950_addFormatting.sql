-- Add rich text formatting column to messages table
-- Stores raw protocol bytes (flag + length + format entries)
ALTER TABLE messages ADD COLUMN formatting BLOB;
