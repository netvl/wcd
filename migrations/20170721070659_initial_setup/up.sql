
CREATE TABLE image_statistics (
  filename TEXT NOT NULL PRIMARY KEY,
  total_displays BIGINT NOT NULL DEFAULT 0,
  total_skips BIGINT NOT NULL DEFAULT 0,
  total_display_seconds BIGINT NOT NULL DEFAULT 0
);
