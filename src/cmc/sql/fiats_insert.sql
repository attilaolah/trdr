INSERT INTO fiats (
    id,
    name,
    sign,
    symbol,
    last_update
) VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (id) DO UPDATE SET
    name = excluded.name,
    sign = excluded.sign,
    symbol = excluded.symbol,
    last_update = excluded.last_update;
