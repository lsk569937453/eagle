CREATE TABLE IF NOT EXISTS user (
	id INTEGER PRIMARY KEY,
	user_id TEXT NOT NULL,
	user_account TEXT NOT NULL,
    user_password TEXT NOT NULL,
    user_authority INTEGER NOT NULL DEFAULT 0,
	timestamp NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now', 'localtime'))
	);
