defmodule Ladefuchs.Repo.Migrations.CreateMspVehicle do
  use Ecto.Migration

  def change do
    create table(:msp) do
      add :name, :string, null: false
      add :enabled, :boolean, null: false, default: true
    end

    create unique_index(:msp, [:name])

    create table(:vehicle) do
      add :name, :string, null: false
    end

    create unique_index(:vehicle, [:name])
  end
end
