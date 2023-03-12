INSERT INTO updates (
    url,
    error_code,
    error_message,
    credit_count,
    timestamp,
    elapsed,
    notice
) VALUES ($1, $2, $3, $4, $5, make_interval(secs => $6), $7)
RETURNING id;
