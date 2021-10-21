defmodule Ladefuchs.TARIFs do
  import Ecto.Query, warn: false
  alias Ladefuchs.Repo

  alias Ladefuchs.TARIFs.TARIF

  def list() do
    TARIF
    |> Repo.all()
  end

  def create(attrs) do
    TARIF.changeset_create(%TARIF{}, attrs)
    |> IO.inspect(label: "changset")
    |> Repo.insert()
  end
end
