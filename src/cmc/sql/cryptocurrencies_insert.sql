INSERT INTO cryptocurrencies (
    id,
    name,
    symbol,
    slug,
    is_active,
    status,
    first_historical_data,
    last_historical_data,
    last_update
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT (id) DO UPDATE SET
    name = excluded.name,
    symbol = excluded.symbol,
    slug = excluded.slug,
    is_active = excluded.is_active,
    status = excluded.status,
    first_historical_data = excluded.first_historical_data,
    last_historical_data = excluded.last_historical_data,
    last_update = excluded.last_update;
