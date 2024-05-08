CREATE TABLE IF NOT EXISTS system_data (
    id INTEGER PRIMARY KEY,
    total_memory INTEGER NOT NULL DEFAULT 0,
    used_memory INTEGER NOT NULL DEFAULT 0,
    total_swap INTEGER NOT NULL DEFAULT 0,
    used_swap INTEGER NOT NULL DEFAULT 0,
    cpu_usage FLOAT NOT NULL,
    disk_usage_read_bytes INTEGER NOT NULL DEFAULT 0,
    disk_usage_written_bytes INTEGER NOT NULL DEFAULT 0,
     network_usage_upload_bytes INTEGER NOT NULL DEFAULT 0,
      network_usage_download_bytes INTEGER NOT NULL DEFAULT 0,
    timestamp NOT NULL DEFAULT (
        strftime('%Y-%m-%d %H:%M:%f', 'now', 'localtime')
    )
);