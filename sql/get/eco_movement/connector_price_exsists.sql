SELECT EXISTS(
    SELECT 1
    FROM eco_movement.connector_price
    WHERE
        location_id = $1
        AND pricing_id = $2
        AND evse_uid = $3
        AND connector_id = $4
) AS "exists!"
