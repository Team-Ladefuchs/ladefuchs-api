defmodule Ladefuchs.MSPs.MSP do
  use Ecto.Schema

  import Ecto.Changeset

  alias __MODULE__, as: MSP

  schema "msp" do
    field :name, :string
    field :enabled, :boolean

    timestamps(type: :utc_datetime)
  end

  def changeset_create(%MSP{} = msp, attrs) do
    msp
    |> cast(attrs, ~w(id name enabled )a)
    |> validate_required(~w(name )a)
    |> validate_length(:name, max: 255)
    |> unique_constraint(:id, name: :msp_pkey)
  end
end
