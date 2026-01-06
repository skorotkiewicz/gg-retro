-- Create messages table for offline message storage
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipient_uin INTEGER NOT NULL,
    sender_uin INTEGER NOT NULL,
    seq INTEGER NOT NULL,
    time INTEGER NOT NULL,
    class INTEGER NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    delivered_at TIMESTAMP
);

-- Composite index for efficient pending message lookups
CREATE INDEX idx_messages_recipient_delivered ON messages (recipient_uin, delivered_at);

-- Index for cleanup queries on delivered messages
CREATE INDEX idx_messages_delivered_at ON messages (delivered_at);
