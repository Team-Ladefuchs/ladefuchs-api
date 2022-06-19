select 
       uuid as id,
       name,
       relationship_id as tariff_id
from vehicle join vehicle_tariff vt
    on vehicle.id = vt.vehicle_id
        join tariff t
            on t.id = vt.tariff_id
where vehicle.is_enabled