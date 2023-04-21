# doggtalk-api
The Core API for DoggTalk

### Enviroument variables

| *KEY*                 | *Required* | *Default* | *Example*                                 |
|-----------------------|------------|-----------|-------------------------------------------|
| WEB_PORT              | N          | 6000      | 6000                                      |
| JWT_SECRET            | Y          |           | secret                                    |
| MYSQL_URL             | Y          |           | mysql://root:root@127.0.0.1:3306/doggtalk |
| MYAQL_MAX_CONNECTIONS | N          | 5         | 10                                        |
| REDIS_URL             | Y          |           | redis://127.0.0.1:6379/0                  |
| REDIS_MAX_CONNECTIONS | N          | 5         | 10                                        |
