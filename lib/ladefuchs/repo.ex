defmodule Ladefuchs.Repo do
  use Ecto.Repo,
    otp_app: :ladefuchs,
    adapter: Ecto.Adapters.Postgres
end
