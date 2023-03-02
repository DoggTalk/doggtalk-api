CREATE TABLE dg_manager (
  id bigint unsigned NOT NULL AUTO_INCREMENT,
  username varchar(100) NOT NULL,
  password varchar(100) NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY UK_username (username)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 AUTO_INCREMENT=1000;

INSERT INTO dg_manager(username,password) VALUES('admin',SHA2('admin', 256));

CREATE TABLE dg_app (
  id bigint unsigned NOT NULL AUTO_INCREMENT,
  app_key varchar(100) NOT NULL,
  app_secret varchar(100) NOT NULL,
  `name` varchar(100) NOT NULL,
  icon_url varchar(256),
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY UK_app_key (app_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 AUTO_INCREMENT=1000;
