defmodule Ladefuchs.CPOs.CPO do
  use Ecto.Schema

  import Ecto.Changeset

  alias __MODULE__, as: CPO

  schema "cpo" do
    field :name, :string
    field :name_slug, :string
    field :enabled, :boolean
    field :types, {:array, :map}, default: []

    timestamps(type: :utc_datetime)
  end

  def changeset_create(%CPO{} = cpo, attrs) do
    cpo
    |> cast(attrs, ~w(id name name_slug enabled types)a)
    |> validate_required(~w(name name_slug)a)
    |> validate_length(:name, max: 255)
    |> validate_length(:name_slug, max: 255)
    |> validate_format(:name_slug, ~r/^[_a-z0-9-]+$/,
      message: "contains illegal characters (allowed: a-z, 0-9, -, and _)"
    )
    |> unique_constraint(:id, name: :cpo_pkey)
  end
end
