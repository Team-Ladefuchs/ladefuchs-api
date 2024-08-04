INSERT INTO app_metrics (app_id, platform, version) VALUES (
    $1, $2, $3
) ON CONFLICT DO NOTHING;
