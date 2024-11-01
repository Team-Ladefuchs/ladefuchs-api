select
    uuid as id,
    name,
    relationship_id as tariff_id
from vehicle inner join vehicle_tariff as vt
    on vehicle.id = vt.vehicle_id
inner join tariff as t
    on vt.tariff_id = t.id
where vehicle.is_enabled = true
