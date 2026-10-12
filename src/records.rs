use rusqlite::{params, Connection, Result};

#[derive(Debug, Clone)]
pub struct Solve {
    pub id: i64,
    pub event_id: String,
    pub time_ms: i64,
    pub scramble: String,
    pub penalty_ms: i64, // 0 = normal, -1 = DNF, positive = penalty in ms
    pub comment: Option<String>,
    pub solved_at: i64,
}

impl Solve {
    pub fn time(&self) -> String {
        if self.penalty_ms < 0 {
            "DNF".to_string()
        } else {
            crate::timer::format_from_ms((self.time_ms + self.penalty_ms.max(0)) as u32)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Average {
    Value(i64),
    Dnf,
    Incomplete,
}

impl Average {
    pub fn to_display(&self) -> String {
        match self {
            Average::Value(t) => crate::timer::format_from_ms(*t as u32),
            Average::Dnf => "DNF".to_string(),
            Average::Incomplete => "---".to_string(),
        }
    }
}

pub fn calc_average(recent_solves: &[Solve], ao: u32) -> Average {
    // Early return: do we have enough solves?
    if recent_solves.len() < ao as usize {
        return Average::Incomplete;
    }
    // Map solves to times, converting DNFs to i64::MAX
    let mut times: Vec<i64> = recent_solves
        .iter()
        .map(|s| if s.penalty_ms < 0 { i64::MAX } else { s.time_ms + s.penalty_ms.max(0) })
        .collect();
    // Sort: real times first, DNFs last
    times.sort();
    // WCA rule: if the second-to-last time is DNF, the whole average is DNF
    // (for Ao5, this means 2 or more DNFs)
    if times[times.len() - 2] == i64::MAX {
        return Average::Dnf;
    }
    // Trimmed sum: discard the best (first) and worst (last) times
    let trimmed_sum: i64 = times[1..times.len() - 1].iter().sum();
    // Return the trimmed mean wrapped in the Average enum
    Average::Value(trimmed_sum / (ao as i64 - 2))
}

pub struct Records {
    conn: Connection,
}

impl Records {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS solves (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL,
                time_ms INTEGER NOT NULL,
                scramble TEXT NOT NULL,
                penalty_ms INTEGER NOT NULL DEFAULT 0,
                comment TEXT,
                solved_at INTEGER NOT NULL
            )",
            (),
        )?;
        Ok(())
    }

    pub fn log(&self, event_id: &str, time_ms: i64, scramble: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO solves (event_id, time_ms, scramble, penalty_ms, solved_at)
             VALUES (?1, ?2, ?3, 0, strftime('%s','now'))",
            params![event_id, time_ms, scramble],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM solves WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn delete_all(&self, event_id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM solves WHERE event_id = ?1", [event_id])?;
        Ok(())
    }

    pub fn set_penalty(&self, id: i64, penalty_ms: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE solves SET penalty_ms = ?1 WHERE id = ?2",
            params![penalty_ms, id],
        )?;
        Ok(())
    }

    pub fn latest(&self, event_id: &str, n: u32) -> Result<Vec<Solve>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, event_id, time_ms, scramble, penalty_ms, comment, solved_at
             FROM solves WHERE event_id = ?1 ORDER BY solved_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![event_id, n], |row| {
            Ok(Solve {
                id: row.get(0)?,
                event_id: row.get(1)?,
                time_ms: row.get(2)?,
                scramble: row.get(3)?,
                penalty_ms: row.get(4)?,
                comment: row.get(5)?,
                solved_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn personal_best(&self, event_id: &str) -> Result<Option<i64>> {
        self.conn.query_row(
            "SELECT MIN(time_ms) FROM solves WHERE event_id = ?1 AND penalty_ms >= 0",
            [event_id],
            |row| row.get(0),
        )
    }

    pub fn average(&self, event_id: &str, ao: u32) -> Result<Average> {
        let solves = self.latest(event_id, ao)?;
        Ok(calc_average(&solves, ao))
    }
}
