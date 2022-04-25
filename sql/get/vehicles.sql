select 
       uuid as id,
       name,
       relationship_id as tarif_id
from vehicle join vehicle_tarif vt
    on vehicle.id = vt.vehicle_id
        join tarif t
            on t.id = vt.tarif_id
where vehicle.is_enabled