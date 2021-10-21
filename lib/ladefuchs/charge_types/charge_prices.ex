defmodule Ladefuchs.ChargePrices do
  import Ecto.Query, warn: false
  alias Ladefuchs.Repo

  alias Ladefuchs.ChargePrices.ChargePrice

  def list() do
    ChargePrice
    |> Repo.all()
  end

  def create(attrs) do
    ChargePrice.changeset_create(%ChargePrice{}, attrs)
    |> IO.inspect(label: "changset")
    |> Repo.insert()
  end
end
