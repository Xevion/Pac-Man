-- Submitted game scores, one row per submission
CREATE TABLE IF NOT EXISTS scores (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  score BIGINT NOT NULL,
  level_count INTEGER NOT NULL,
  duration_ms INTEGER NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Ranking is by score descending; the leaderboard reads each user's best.
CREATE INDEX IF NOT EXISTS idx_scores_score ON scores (score DESC);
CREATE INDEX IF NOT EXISTS idx_scores_user_score ON scores (user_id, score DESC);
CREATE INDEX IF NOT EXISTS idx_scores_created_at ON scores (created_at);
