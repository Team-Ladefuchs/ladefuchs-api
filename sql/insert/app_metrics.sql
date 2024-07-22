INSERT INTO app_metrics(app_id, platform) VALUES ($1, $2) on conflict do nothing;
