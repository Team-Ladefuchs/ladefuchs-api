import Config

# Configure your database
config :ladefuchs, Ladefuchs.Repo,
  username: System.get_env("POSTGRES_USER") || "adminfuchs",
  password: System.get_env("POSTGRED_PASSWORD") || "ringdingdingdingdingading",
  database: System.get_env("POSTGRES_DB") || "ladefuchs",
  hostname: "localhost",
  port: System.get_env("POSTGRES_PORT") || "54320",
  pool: Ecto.Adapters.SQL.Sandbox,
  pool_size: 10

# We don't run a server during test. If one is required,
# you can enable the server option below.
config :ladefuchs, LadefuchsWeb.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4002],
  server: false

# Print only warnings and errors during test
config :logger, level: :warn

# Initialize plugs at runtime for faster test compilation
config :phoenix, :plug_init_mode, :runtime
