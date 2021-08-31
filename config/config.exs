# This file is responsible for configuring your application
# and its dependencies with the aid of the Config module.
#
# This configuration file is loaded before any dependency and
# is restricted to this project.

# General application configuration
import Config

config :ladefuchs,
  ecto_repos: [Ladefuchs.Repo]

# Configures the endpoint
config :ladefuchs, LadefuchsWeb.Endpoint,
  url: [host: "localhost"],
  secret_key_base: "omndRPfXMsgUf8GFLHALVnb8MTWSbH5aI6ao/nOzs6Fq7ST7Mdw+1FdrtD0bjj73",
  render_errors: [view: LadefuchsWeb.ErrorView, accepts: ~w(json), layout: false],
  pubsub_server: Ladefuchs.PubSub,
  live_view: [signing_salt: "qgMCRDZB"]

# Configures Elixir's Logger
config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

# Use Jason for JSON parsing in Phoenix
config :phoenix, :json_library, Jason

# Import environment specific config. This must remain at the bottom
# of this file so it overrides the configuration defined above.
import_config "#{config_env()}.exs"
