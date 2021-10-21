defmodule Ladefuchs.ChargePrices.ChargePrice do
  use Ecto.Schema

  import Ecto.Changeset

  alias __MODULE__, as: ChargePrice

  schema "charge_price" do
    field :tarif_id, :integer
    field :cpo_id, :integer
    field :type, Ecto.Enum, values: [:AC, :DC]
    field :price, :integer, default: 0
    field :blocking_fee_start, :integer, default: 0
    field :updated, :utc_datetime

    timestamps(type: :utc_datetime)
  end

  def changeset_create(%ChargePrice{} = charge_price, attrs) do
    charge_price
    |> cast(attrs, ~w(id)a)
    |> validate_required(~w(name)a)
    |> validate_length(:name, max: 255)
    |> validate_number(:price, greater_than: 0)
    |> validate_number(:blocking_fee_start, greater_than: 0)
    |> unique_constraint(:id)
  end
end
