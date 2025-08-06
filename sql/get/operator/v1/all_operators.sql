SELECT
    operator.slug_name AS display_name,
    CONCAT('cpo-', LOWER(operator.name)) AS "identifier!",
    LOWER(operator.name) AS "name!"
FROM
    operator
WHERE
    EXISTS (
        SELECT 1 FROM charge_price
        WHERE operator_id = id AND charge_price.is_protected = false
    )
    AND $1 != ''
ORDER BY
    operator.name;
