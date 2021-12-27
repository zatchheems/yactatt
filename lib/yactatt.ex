defmodule Yactatt do
  @moduledoc """
  Yet Another CTA Transit Tracker.
  
  The main goal is to eventually drive an LED matrix and prettily display exactly how fucking late
  I'll be to literally everything in Chicago all the goddamned time. And I'll want to see whether
  buses and trains will actually show up or just vaporize into thin air-- that is,
  I'd like to see a visual distinction between scheduled buses and physically present ones.

  This is a fun project made purely out of spite and a desire to do something fun
  with Nerves and Raspberry Pi.

  Be kind to your drivers. They do not get paid enough to deal with this shit.
  """

  @cta_endpoint "http://www.ctabustracker.com"
  @bustracker_path "/bustime/api/v2/getvehicles"

  @doc """
  Get buses based on the routes provided.
  Bear in mind the CTA API only allows for 10 routes to be specified at once.

  ## Examples

    iex> Yactatt.get_buses("49")
    {:ok, %{"49" => %{some: "data"}}}
  
    iex> Yactatt.get_buses("157", "8", "12", "18")
    {
      :ok,
      %{
        "157" => %{some: "data"},
        "50" => %{some: "data"},
        "136" => %{some: "data"},
        "18" => %{some: "data"},
      }
    }

  """
  def get_buses(route) when is_binary(route), do: route |> List.wrap() |> get_buses()
  def get_buses(routes) when is_list(routes) and length(routes) > 0 and length(routes) <= 10 do
    routes_str = Enum.map_join(routes,",",&(&1))
    IO.inspect routes_str

    key = System.get_env("CTA_BUSTRACKER_KEY") #Application.get_env(:yactatt, :cta_bustracker_key)

    :httpc.request(@cta_endpoint <> @bustracker_path <> "?format=json&key=" <> key <> "&rt=" <> routes_str)
  end
end
