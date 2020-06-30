CREATE TABLE reminders (
  uid INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  -- JSON object of day name to an array of the times of day
  schedule TEXT NOT NULL,
  -- Beginning of the week in which the schedule started
  startweek INTEGER NOT NULL,
  -- JSON array of integer user uids, in order of assignment
  assignees TEXT NOT NULL
)
