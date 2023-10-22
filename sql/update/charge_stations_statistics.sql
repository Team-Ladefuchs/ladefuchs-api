UPDATE operator
SET ccs_plug_count = $2,
    type2_plug_count = $3
WHERE network = $1
