defmodule Ladefuchs.MSPs do
  import Ecto.Query, warn: false
  alias Ladefuchs.Repo

  alias Ladefuchs.MSPs.MSP

  def list() do
    MSP
    |> Repo.all()
  end

  def create(attrs) do
    MSP.changeset_create(%MSP{}, attrs)
    |> IO.inspect(label: "changset")
    |> Repo.insert()
  end
end
