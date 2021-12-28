defmodule Yactatt do
  @moduledoc """
  Yet Another CTA Transit Tracker.

  Get buses and trains by route number/color.

  TODO: trains.

  TODO: proximity-based bus tracking, wherein if the heading is pointing towards
  the device's location and approaching, the associated buses and trains will be retrieved.
  """

  @cta_endpoint "http://www.ctabustracker.com"
  @bustracker_path "/bustime/api/v2/getvehicles"

  @doc """
  Get buses based on the routes provided.
  Bear in mind the CTA API only allows for 10 routes to be specified at once.

  ## Examples

    iex> Yactatt.get_buses("49")
    {:ok,
      %{"49" => [
        %Vehicle{
        delayed: false,
        destination: "Berwyn",
        heading: "8",
        latitude: "41.833227920532224",
        longitude: "-87.68491668701172",
        pattern_distance: 30239,
        pattern_id: 1180,
        route: "49",
        timestamp: ~N[2021-12-27 17:21:00]
      }]
    }}
  
    iex> Yactatt.get_buses(["157", "8", "12", "18"])
    {
      :ok,
      %{
        "8" => %{some: "data"},
        "18" => %{some: "data"},
        "49" => %{some: "data"},
        "146" => %{some: "data"},
      }
    }

  """
  @spec get_buses([String.t()]) :: [Vehicle.t()]
  def get_buses(route) when is_binary(route), do: route |> List.wrap() |> get_buses()
  def get_buses(routes) when is_list(routes) and length(routes) > 0 and length(routes) <= 10 do
    routes_str = Enum.map_join(routes,",",&(&1))
    IO.inspect routes_str

    key = System.get_env("CTA_BUSTRACKER_KEY", "") #Application.get_env(:yactatt, :cta_bustracker_key)

    with {:ok, {_http_response, _headers, response}} <- :httpc.request(@cta_endpoint <> @bustracker_path <> "?format=json&key=" <> key <> "&rt=" <> routes_str),
         {:ok, %{"bustime-response" => %{"vehicle" => bustimes}}} <- Jason.decode(response) do
      Enum.map(bustimes, &(Vehicle.map_response_to_struct(&1)))
    end
  end

  # TODO: based on current coordinates, determine closest bus stops and look for those
end

defmodule Vehicle do
  @enforce_keys [:destination, :heading, :latitude, :longitude, :route]
  defstruct [
    delayed: false,
    destination: "",
    heading: "0",        # Direction bus is facing in degrees, where 0/360 is north
    latitude: "",
    longitude: "",
    pattern_distance: 0, # distance bus has traveled along path
    pattern_id: 0,       # ID of path
    route: "",           # route number
    timestamp: NaiveDateTime.utc_now()
  ]
  @type t :: %__MODULE__{delayed: boolean(), destination: String.t(), heading: String.t(), latitude: String.t(), longitude: String.t(), pattern_distance: non_neg_integer, pattern_id: non_neg_integer, route: String.t()}

  defp timestamp_str_to_naive_datetime(str) do
    [date, time] = String.split(str, " ")
    [hour_str, minute_str] = String.split(time, ":")

    year = binary_part(date, 0, 4) |> String.to_integer()
    month = binary_part(date, 4, 2) |> String.to_integer()
    day = binary_part(date, 6, 2) |> String.to_integer()

    hour = String.to_integer(hour_str)
    minute = String.to_integer(minute_str)

    NaiveDateTime.from_erl!({{year, month, day}, {hour, minute, 00}})
  end

  def map_response_to_struct(vehicle) when is_map(vehicle) do
    %{
      "des" => destination,
      "dly" => delayed,
      "hdg" => heading,
      "lat" => latitude,
      "lon" => longitude,
      "pdist" => pattern_distance,
      "pid" => pattern_id,
      "rt" => route,
      "tmstmp" => timestamp_str,
    } = vehicle
    timestamp = timestamp_str_to_naive_datetime(timestamp_str)
    struct(__MODULE__,
      delayed: delayed,
      destination: destination,
      heading: heading,
      latitude: latitude,
      longitude: longitude,
      pattern_distance: pattern_distance,
      pattern_id: pattern_id,
      route: route,
      timestamp: timestamp
    )
  end
end
