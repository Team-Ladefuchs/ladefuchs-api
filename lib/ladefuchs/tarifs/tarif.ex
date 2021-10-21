defmodule Ladefuchs.TARIFs.TARIF do
  use Ecto.Schema

  import Ecto.Changeset

  alias __MODULE__, as: TARIF

  schema "tarif" do
    field :name, :string

    timestamps(type: :utc_datetime)
  end

  def changeset_create(%TARIF{} = tarif, attrs) do
    tarif
    |> cast(attrs, ~w(id name )a)
    |> validate_required(~w(name )a)
    |> validate_length(:name, max: 255)
    |> unique_constraint(:id, name: :tarif_pkey)
  end
end
