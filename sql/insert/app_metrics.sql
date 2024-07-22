INSERT INTO app_metrics(app_id, platform, version) VALUES ($1, $2, $3) on conflict do nothing;
