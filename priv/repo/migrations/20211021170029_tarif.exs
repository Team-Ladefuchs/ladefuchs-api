defmodule Ladefuchs.Repo.Migrations.Tarif do
  use Ecto.Migration

  def change do
    create table(:tarif) do
      add :name, :string, null: false
      add :monhtly_fee, :integer, null: false
      add :msp_id, references(:msp), null: false
      add :vehicle_id, references(:vehicle), null: false
    end

    create unique_index(:tarif, [:name])
  end
end
