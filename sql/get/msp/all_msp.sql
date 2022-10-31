select
  distinct on (pub_msp_id, msp.name) msp.name,
  pub_msp_id as id,
  c.pub_network as operator_id
from
  msp
  join msp_cpo mc on msp.id = mc.msp_id
  join cpo c on c.id = mc.cpo_id
where
  c.is_enabled
order by
  msp.name
