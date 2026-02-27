select coalesce(sum(count), 0) from affiliate_statistic_daily where link_id = $1
