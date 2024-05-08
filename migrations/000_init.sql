CREATE TABLE IF NOT EXISTS system (
    id INTEGER PRIMARY KEY,
    total_memory INTEGER NOT NULL,
    used_memory INTEGER NOT NULL,
    total_swap INTEGER NOT NULL,
    used_swap INTEGER NOT NULL,
    cpu_usage TEXT NOT NULL,
    network_usage INTEGER NOT NULL DEFAULT 0,
    timestamp NOT NULL DEFAULT (
        strftime('%Y-%m-%d %H:%M:%f', 'now', 'localtime')
    )
);