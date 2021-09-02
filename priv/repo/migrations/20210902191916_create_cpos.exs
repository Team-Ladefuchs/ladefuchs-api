defmodule Ladefuchs.Repo.Migrations.CreateCpos do
  use Ecto.Migration

  def change do
    create table(:cpo) do
      add :name, :string, null: false
      add :name_slug, :string, null: false
      add :enabled, :boolean, null: false, default: true
      add :types, :map, null: false, default: fragment("'[]'")

      timestamps(type: :utc_datetime)
    end

    create unique_index(:cpo, [:name_slug])

  end
end
