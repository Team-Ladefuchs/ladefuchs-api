INSERT INTO vehicle_tarif(vehicle_id, tarif_id) 
VALUES($1, $2) 
ON CONFLICT(vehicle_id, tarif_id) do nothing;