INSERT INTO feedback (
    operator_id, tariff_id, language, notes, kind, context
)
VALUES ($1, $2, $3, $4, $5, $6)
