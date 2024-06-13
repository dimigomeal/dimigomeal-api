use rusqlite::Error;

use rusqlite::Connection;

pub struct Meal {
    pub idx: i32,
    pub id: i32,
    pub date: String,
    pub breakfast: String,
    pub lunch: String,
    pub dinner: String,
}

pub fn create_conn() -> Connection {
    let db_path = "./db.db3";
    let conn = Connection::open(db_path).unwrap();

    conn
}

pub fn get_meal(conn: &Connection, target_date: &str) -> Result<Meal, Error> {
    let meal = conn.query_row("SELECT * FROM meals WHERE date=?1", [target_date], |row| {
        Ok(Meal {
            idx: row.get(0)?,
            id: row.get(1)?,
            date: row.get(2)?,
            breakfast: row.get(3)?,
            lunch: row.get(4)?,
            dinner: row.get(5)?,
        })
    });

    meal
}

pub fn get_multi_meals(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<Meal>, Error> {
    let mut stmt = conn.prepare("SELECT * FROM meals WHERE date BETWEEN ?1 AND ?2")?;
    let meals = stmt.query_map([start_date, end_date], |row| {
        Ok(Meal {
            idx: row.get(0)?,
            id: row.get(1)?,
            date: row.get(2)?,
            breakfast: row.get(3)?,
            lunch: row.get(4)?,
            dinner: row.get(5)?,
        })
    })?;

    let mut meal_list = Vec::new();
    for meal in meals {
        meal_list.push(meal.unwrap());
    }

    Ok(meal_list)
}
