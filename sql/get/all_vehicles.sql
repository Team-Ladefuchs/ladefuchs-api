select id, uuid, name, vehicle_type::VehicleType
from vehicle
where is_enabled = true