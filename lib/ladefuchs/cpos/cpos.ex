defmodule Ladefuchs.CPOs do
  import Ecto.Query, warn: false
  alias Ladefuchs.Repo

  alias Ladefuchs.CPOs.CPO

  def list() do
    CPO
    |> Repo.all()
  end

  def create(attrs) do
    CPO.changeset_create(%CPO{}, attrs)
    |> IO.inspect(label: "changset")
    |> Repo.insert()
  end
end
