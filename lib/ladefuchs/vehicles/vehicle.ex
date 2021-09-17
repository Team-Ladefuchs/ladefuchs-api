defmodule Ladefuchs.VEHICLEs.VEHICLE do
  use Ecto.Schema

  import Ecto.Changeset

  alias __MODULE__, as: VEHICLE

  schema "vehicle" do
    field :vehicle, :string

    timestamps(type: :utc_datetime)
  end

  def changeset_create(%VEHICLE{} = vehicle, attrs) do
    vehicle
    |> cast(attrs, ~w(id name )a)
    |> validate_required(~w(name )a)
    |> validate_length(:name, max: 255)
    |> unique_constraint(:id, name: :vehicle_pkey)
  end
end
