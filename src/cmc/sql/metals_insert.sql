INSERT INTO metals (
    id,
    name,
    code,
    unit,
    last_update
) VALUES ($1, $2, $3, $4::metal_unit, $5)
ON CONFLICT (id) DO UPDATE SET
    name = excluded.name,
    code = excluded.code,
    unit = excluded.unit,
    last_update = excluded.last_update;
