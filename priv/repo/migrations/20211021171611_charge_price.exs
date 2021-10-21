defmodule Ladefuchs.Repo.Migrations.ChargePrice do
  use Ecto.Migration
  alias Ladefuchs.Charge_Type
  def change do

    create table(:charge_price, primary_key: false) do
      add :price, :integer, null: false
      add :blocking_fee_start, :integer, null: false
      add :updated, :utc_datetime, null: false
      # type is an enum [AC, DC]
      add :type, :string, null: false
      add :cpo_id, references(:cpo), primary_key: true
      add :tarif, references(:tarif), primary_key: true
    end

  end
end
