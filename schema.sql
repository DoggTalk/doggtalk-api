CREATE TABLE dg_managers (
  id bigint unsigned NOT NULL AUTO_INCREMENT,
  username varchar(100) NOT NULL,
  password varchar(100) NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY UK_username (username)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 AUTO_INCREMENT=1000;

INSERT INTO dg_managers(username,password) VALUES('admin',SHA2('admin', 256));

CREATE TABLE dg_apps (
  id bigint unsigned NOT NULL AUTO_INCREMENT,
  app_key varchar(100) NOT NULL,
  app_secret varchar(100) NOT NULL,
  `name` varchar(100) NOT NULL,
  icon_url varchar(256),
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY UK_app_key (app_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 AUTO_INCREMENT=1000;

/*
    source - 0-Fake,1-Sync
    status - 0-Pendding,1-Active,2-Ban
*/

CREATE TABLE dg_users (
  id bigint unsigned NOT NULL AUTO_INCREMENT,
  app_id bigint unsigned NOT NULL,
  `source` tinyint NOT NULL,
  account varchar(128) NOT NULL,
  display_name varchar(128) NOT NULL,
  status tinyint NOT NULL,
  avatar_url varchar(2048),
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY UK_user_account (app_id,`source`,account)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 AUTO_INCREMENT=1000;

/*
    topped - 0-normal, >0 topped, <0 deleted
*/

CREATE TABLE dg_topics (
  id bigint unsigned NOT NULL AUTO_INCREMENT,
  app_id bigint unsigned NOT NULL,
  user_id bigint unsigned NOT NULL,
  category bigint unsigned NOT NULL,
  title varchar(1024) NOT NULL,
  content text NOT NULL,
  topped bigint unsigned NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  refreshed_at DATETIME NOT NULL,
  reply_count bigint unsigned NOT NULL,
  PRIMARY KEY (id),
  KEY IX_topic_user (user_id,topped),
  KEY IX_topic_create (app_id,category,topped,created_at),
  KEY IX_topic_refresh (app_id,category,topped,refreshed_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 AUTO_INCREMENT=1000;


/*
    topped - 0-normal, >0 topped, <0 deleted
*/

CREATE TABLE dg_replies (
  id bigint unsigned NOT NULL AUTO_INCREMENT,
  app_id bigint unsigned NOT NULL,
  topic_id bigint unsigned NOT NULL,
  user_id bigint unsigned NOT NULL,
  content text NOT NULL,
  topped bigint unsigned NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY IX_reply_user (user_id,topped),
  KEY IX_reply_create (topic_id,topped,created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 AUTO_INCREMENT=1000;
