pub fn insert_into(table: &str, fields: &[&str]) -> String {
    let placeholders: Vec<String> = (1..fields.len() + 1).map(|i| format!("${}", i)).collect();

    format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        fields.join(", "),
        placeholders.join(", "),
    )
}

pub fn upsert_into(table: &str, fields: &[&str], unique: &str) -> String {
    let excl: Vec<String> = fields
        .into_iter()
        .filter(|f| *f != &unique)
        .map(|f| format!("{} = excluded.{}", f, f))
        .collect();

    format!(
        "{} ON CONFLICT ({}) DO UPDATE SET {}",
        insert_into(table, fields),
        unique,
        excl.join(", ")
    )
}
