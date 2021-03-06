#![macro_use]

use super::*;

#[macro_export]
macro_rules! mock_server_connection {
    ($struct_name:ident, $ret_val:expr) => {
        struct $struct_name;
        impl ServerConnection for $struct_name {
            fn get_game_data(_: &str) -> io::Result<GameData> {
                $ret_val
            }
        }
    }
}

#[macro_export]
macro_rules! mock_conditional_server_connection {
    ($struct_name:ident, $ret_fn:expr) => {
        struct $struct_name;
        impl ServerConnection for $struct_name {
            fn get_game_data(server_address: &str) -> io::Result<GameData> {
                $ret_fn(server_address)
            }
        }
    }
}

use std::error::Error;
fn trace_fn(x: &str) { println!("TRACE: {:?}", x); }
impl DbConnection {
    fn test_initialise(&mut self) -> Result<(), Box<Error>> {
        let conn = &mut *self.0.clone().get()?;
        conn.trace(Some(trace_fn));
        let tx = conn.transaction()?;
        for ref migration in MIGRATIONS.as_ref() {
            tx.execute_batch(migration.up.unwrap())?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn test() -> Self {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder().max_size(1).build(manager).unwrap();
        let mut db_conn = DbConnection(pool);
        let result = db_conn.test_initialise();
        println!("TEST DB INITIALISATION: {:?}", result);
        result.unwrap();
        db_conn
    }

    pub fn noop() -> Self {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::new(manager).unwrap();
        DbConnection(pool)
    }

    pub fn count_servers(&self) -> i32 {
        let conn = &*self.0.clone().get().unwrap();
        conn.query_row("SELECT COUNT(*) FROM game_servers", &[], |r| r.get(0))
            .unwrap()
    }

    pub fn count_started_server_state(&self) -> i32 {
        let conn = &*self.0.clone().get().unwrap();
        conn.query_row("SELECT COUNT(*) FROM started_servers", &[], |r| r.get(0))
            .unwrap()
    }

    pub fn count_lobby_state(&self) -> i32 {
        let conn = &*self.0.clone().get().unwrap();
        conn.query_row("SELECT COUNT(*) FROM lobbies", &[], |r| r.get(0))
            .unwrap()
    }
}

/*
RUNNING: Some("\r\ncreate table if not exists players (\r\n
id INTEGER NOT NULL PRIMARY KEY,\r\n    discord_user_id int NOT NULL,\r\n
  turn_notifications BOOLEAN NOT NULL,\r\n    CONSTRAINT discord_user_id_unique U
  NIQUE(discord_user_id)\r\n);\r\n\r\ncreate table if not exists started_servers (\r\n
    id INTEGER NOT NULL PRIMARY KEY,\r\n    address VARCHAR(255) NOT NULL,\r\n    last_seen_turn int NOT NULL,\r\n    CONSTRAINT server_address_unique UNIQUE (address)\r\n);\r\n\r\ncreate table if not exists lobbies (\r\n    id INTEGER NOT NULL PRIMARY KEY,\r\n    owner_id int NOT NULL REFERENCES players(id),\r\n    player_count int NOT NULL,\r\n    era int NOT NULL\r\n);\r\n\r\ncreate table if not exists game_servers (\r\n    id INTEGER NOT NULL PRIMARY KEY,\r\n    alias VARCHAR(255) NOT NULL,\r\n\r\n    started_server_id int REFERENCES started_servers(id),\r\n    lobby_id int REFERENCES lobbies(id),\r\n\r\n    CONSTRAINT server_alias_unique UNIQUE (alias)\r\n);\r\n\r\ncreate table if not exists server_players (\r\n    server_id int NOT NULL REFERENCES game_servers(id),\r\n    player_id int NOT NULL REFERENCES players(id),\r\n    nation_id int NOT NULL,\r\n\r\n    CONSTRAINT server_nation_unique UNIQUE (server_id, nation_id)\r\n);\r\n")

*/