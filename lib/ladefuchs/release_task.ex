# Idea taken from http://blog.plataformatec.com.br/2016/04/running-migration-in-an-exrm-release/ (note that we use
# elixir's release task instead of exrm, though!)
defmodule Ladefuchs.ReleaseTasks do
  require Logger

  def migrate() do
    Logger.info("Start migrate task")

    {:ok, _} = Application.ensure_all_started(:ladefuchs)

    path = Application.app_dir(:ladefuchs, "priv/repo/migrations")

    Logger.info("Run ecto migration")
    Ecto.Migrator.run(Ladefuchs.Repo, path, :up, all: true)

    :init.stop()

    Logger.info("Finished migrate task")
  end
end
