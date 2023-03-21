use tokio_postgres::types::ToSql;

pub const MAX_PARAMS: usize = 32_767;

type SqlVal = dyn ToSql + Sync;
pub type SqlVals<'a> = Vec<&'a SqlVal>;

pub fn insert_into(table: &str, fields: &[&str], rows: usize) -> String {
    let placeholders: Vec<String> = (0..rows)
        .map(|row| {
            let cols: Vec<String> = (1..fields.len() + 1)
                .map(|i| format!("${}", fields.len() * row + i))
                .collect();
            format!("({})", cols.join(", "))
        })
        .collect();

    format!(
        "INSERT INTO {} ({}) VALUES {}",
        table,
        fields.join(", "),
        placeholders.join(", "),
    )
}

pub fn upsert_into(table: &str, fields: &[&str], rows: usize, unique: &str) -> String {
    let excl: Vec<String> = fields
        .into_iter()
        .filter(|f| *f != &unique)
        .map(|f| format!("{} = excluded.{}", f, f))
        .collect();

    format!(
        "{} ON CONFLICT ({}) DO UPDATE SET {}",
        insert_into(table, fields, rows),
        unique,
        excl.join(", ")
    )
}
