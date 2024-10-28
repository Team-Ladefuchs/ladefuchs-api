SELECT
    vehicle.uuid AS id,
    vehicle.name,
    relationship_id AS tariff_id
FROM
    vehicle
LEFT JOIN
    vehicle_tariff vt ON vehicle.id = vt.vehicle_id
LEFT JOIN
    tariff t ON t.id = vt.tariff_id
WHERE
    vehicle.is_enabled  OR vt.tariff_id IS NULL
