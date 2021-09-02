defmodule LadefuchsWeb.Router do
  use LadefuchsWeb, :router

  pipeline :api do
    plug :accepts, ["json"]
  end

  pipeline :apidocs do
    plug Plug.Static,
      at: "/apidocs",
      brotli: true,
      from: {:service_feeds, "priv/static/apidocs"}
  end

  scope "/api", LadefuchsWeb do
    pipe_through :api
  end

  scope "/apidocs", LadefuchsWeb do
    pipe_through :apidocs

    get "/*path", Redirect, to: "/apidocs/index.html"
  end

  # Enables LiveDashboard only for development
  #
  # If you want to use the LiveDashboard in production, you should put
  # it behind authentication and allow only admins to access it.
  # If your application does not have an admins-only section yet,
  # you can use Plug.BasicAuth to set up some basic authentication
  # as long as you are also using SSL (which you should anyway).
  if Mix.env() in [:dev, :test] do
    import Phoenix.LiveDashboard.Router

    scope "/" do
      pipe_through [:fetch_session, :protect_from_forgery]
      live_dashboard "/dashboard", metrics: LadefuchsWeb.Telemetry
    end
  end
end
