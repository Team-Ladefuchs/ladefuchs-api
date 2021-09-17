defmodule Ladefuchs.VEHICLEs do
  import Ecto.Query, warn: false
  alias Ladefuchs.Repo

  alias Ladefuchs.VEHICLEs.VEHICLE

  def list() do
    VEHICLE
    |> Repo.all()
  end

  def create(attrs) do
    VEHICLE.changeset_create(%VEHICLE{}, attrs)
    |> IO.inspect(label: "changset")
    |> Repo.insert()
  end
end
