SELECT
    operator.slug_name AS display_name,
    CONCAT('cpo-', LOWER(operator.name)) AS "identifier!",
    LOWER(operator.name) AS "name!"
FROM
    operator
WHERE
    EXISTS (
        SELECT operator_id FROM charge_price
        WHERE operator_id = id AND charge_price.is_protected = false
    )
    AND (operator.standard = $1 AND $2 != '')
ORDER BY
    operator.name;
