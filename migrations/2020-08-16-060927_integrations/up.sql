CREATE TABLE integrations (
  uid INTEGER NOT NULL,
  uid_type TEXT NOT NULL,
  name TEXT NOT NULL,
  data TEXT NOT NULL,
  PRIMARY KEY(uid, uid_type, name)
)
