defmodule Yactatt do
  @moduledoc """
  Get buses and trains by route number/color.
  Data provided by Chicago Transit Authority.

  TODO: trains.

  TODO: proximity-based bus tracking, wherein if the heading is pointing towards
  the device's location and approaching, the associated buses and trains will be retrieved.
  There is also a possibility of writing a config file and reading from that as a sort of "calibration"
  step. I may want to do this in the Nerves project instead of this library as a separation of concerns.

  TODO: reconcile predictions with actual GPS locations for buses.
  """

  @bustracker_endpoint "http://www.ctabustracker.com"
  @bustracker_vehicles_path "/bustime/api/v2/getvehicles"
  @bustracker_predictions_path "/bustime/api/v2/getpredictions"

  @traintracker_endpoint "http://lapi.transitchicago.com"
  @traintracker_path "/api/1.0/ttarrivals.aspx"

  require Logger

  @spec is_non_neg_integer(any) ::
          {:__block__ | {:., [], [:andalso | :erlang, ...]}, [],
           [{:= | {any, any, any}, [], [...]}, ...]}
  defguard is_non_neg_integer(int) when is_integer(int) and int > 0

  # NOTE: uses builtin Erlang httpc library, which doesn't validate SSL certs.
  # The API doesn't use HTTPS, so this is fine for now, if a bit quick and dirty.
  defp bus_api_request(args, type \\ :vehicles) do
    key = Application.fetch_env!(:yactatt, :cta_bustracker_key)
    path = case type do
      :vehicles -> @bustracker_vehicles_path
      :predictions -> @bustracker_predictions_path
    end
    url = @bustracker_endpoint <> path <> "?format=json&key=" <> key <> args
    Logger.debug url
    with {:ok, {_http_response, _headers, response}} <- :httpc.request(url),
         {:ok, %{"bustime-response" => %{"vehicle" => bustimes}}} <- Jason.decode(response) do
      {:ok, Enum.map(bustimes, &(CTABus.map_response_to_struct(&1)))}
    else
      {:ok, %{"bustime-response" => %{"error" => [%{"msg" => "No data found for parameter", "rt" => _}]}}} ->
        {:error, "No buses found with given input."}
      result -> Logger.error(result)
        {:error, result}
    end
  end

  @spec get_buses!([{:routes, [non_neg_integer] | pos_integer}, ...]) :: list
  @doc """
  Get buses based on the routes provided.
  Bear in mind the CTA API only allows for 10 routes to be specified at once.

  ## Examples

    iex> Yactatt.get_buses(routes: 49)
    {:ok,
      %{"49" => [
        %CTABus{
          is_delayed: false,
          destination: "Berwyn",
          heading: "8",
          latitude: "41.833227920532224",
          longitude: "-87.68491668701172",
          pattern_distance: 30239,
          pattern_id: 1180,
          route: "49",
          timestamp: ~N[2021-12-27 17:21:00]
        }
      ]
    }}

    ### With a list of bus routes:

    iex> Yactatt.get_buses(routes: [157, 8, 12, 18])
    {
      :ok,
      %{
        "8" => %CTABus{some: "data"},
        "18" => %CTABus{some: "data"},
        "49" => %CTABus{some: "data"},
        "146" => %CTABus{some: "data"}
      }
    }

    ### With a stop ID or list of stops:
    iex> Yactatt.get_buses(stops: [8059, 8388, 548, 15203])
    {
      :ok,
      %{
        "8058" => %CTABus{some: "more data"},
        "8388" => %CTABus{some: "more data"},
        "548" => %CTABus{some: "more data"},
        "15203" => %CTABus{some: "more data"}
      }
    }

  TODO: allow more specfic requests using keyword lists, e.g. routes: [1,2,3], stops: [4321,4231]
  """
  @spec get_buses(routes: non_neg_integer | [non_neg_integer]) :: {atom, [Vehicle.t()] | String.t()}
  def get_buses!(args) do
    {:ok, buses} = get_buses(args)
    buses
  end
  def get_buses(route) when not is_list(route) and is_non_neg_integer(route), do: get_buses(routes: List.wrap(route))
  def get_buses(routes: route) when not is_list(route) and is_non_neg_integer(route), do: get_buses(routes: List.wrap(route))
  def get_buses(routes: routes) when is_list(routes) and length(routes) > 0 and length(routes) <= 10 do
    valid_routes = Enum.filter(routes, &(is_non_neg_integer(&1)))
    if length(valid_routes) == 0, do: raise ArgumentError, message: ~s(No valid bus routes given.\n\n  Bus routes are individuals or lists of non negative integers, \n  ex: 146, [1,2,3,4], etc.\n)

    routes_str = Enum.map_join(valid_routes,",", &(Integer.to_string(&1)))

    bus_api_request("&rt=" <> routes_str)
  end

  @spec get_bus_predictions([{:routes, list} | {:stops, list}, ...]) :: {:error, any} | {:ok, list}
  def get_bus_predictions(stops: stops) when is_list(stops) and length(stops) > 0 and length(stops) <= 10 do
    valid_stops = Enum.filter(stops, &(is_non_neg_integer(&1)))
    if length(valid_stops) == 0, do: raise ArgumentError, message: ~s(No valid bus stops given.\n\n  Bus stops are individuals or lists of non negative integers, \n  ex: 8825, [15203,11003], etc.\n)

    stops_str = Enum.map_join(valid_stops,",", &(Integer.to_string(&1)))

    bus_api_request("&stpid=" <> stops_str, :predictions)
  end

  @doc """
  Get trains based on stop(s) provided.

  ## Examples

  TODO.

  """
  # FIXME: not a very useful function. Also, add @spec
  def get_trains() do
    key = Application.get_env(:yactatt, :cta_traintracker_key)
    "?outputType=JSON&key=#{key}&mapid=40380"
    with {:ok, {_http_response, _headers, response}} <- :httpc.request(@traintracker_endpoint <> @traintracker_path <> "?outputType=JSON&key=#{key}&mapid=40380"),
         {:ok, %{"ctatt" => %{"errCd" => _, "errNm" => _, "eta" => traintimes, "tmst" => _}}} <- Jason.decode(response) do
      {:ok, Enum.map(traintimes, &(CTATrain.map_response_to_struct(&1)))}
    end
  end
  # TODO: based on current coordinates, determine closest bus/train stops and look for those
end

defmodule CTABus do
  @enforce_keys [:destination, :heading, :latitude, :longitude, :route]
  defstruct [
    destination: "",
    heading: "0",        # Direction bus is facing in degrees, where 0/360 is north
    is_delayed: false,
    latitude: "",
    longitude: "",
    pattern_distance: 0, # distance bus has traveled along path
    pattern_id: 0,       # ID of path
    route: "",           # route number
    timestamp: NaiveDateTime.utc_now()
  ]
  @type t :: %__MODULE__{is_delayed: boolean(), destination: String.t(), heading: String.t(), latitude: String.t(), longitude: String.t(), pattern_distance: non_neg_integer, pattern_id: non_neg_integer, route: String.t()}

  defp timestamp_str_to_datetime(str) do
    [date, time] = String.split(str, " ")

    year = binary_part(date, 0, 4)
    month = binary_part(date, 4, 2)
    day = binary_part(date, 6, 2)
    # FIXME: incorrect time zone offset. Should be UTC-06:00
    {:ok, timestamp_str, _} = DateTime.from_iso8601(~s(#{year}-#{month}-#{day}T#{time}:00-06))

    timestamp_str
  end

  @spec map_response_to_struct(map) :: struct
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
    timestamp = timestamp_str_to_datetime(timestamp_str)
    struct(__MODULE__,
      destination: destination,
      heading: heading,
      is_delayed: delayed,
      latitude: latitude,
      longitude: longitude,
      pattern_distance: pattern_distance,
      pattern_id: pattern_id,
      route: route,
      timestamp: timestamp
    )
  end
end

defmodule CTATrain do
  @train_routes %{
    "Blue" => :blue,
    "Brn" => :brown,
    "G" => :green,
    "Org" => :orange,
    "Pink" => :pink,
    "P" => :purple,
    "Red" => :red,
    "Y" => :yellow
  }

  @enforce_keys [:destination_name, :heading, :latitude, :longitude, :route, :station_name]
  defstruct [
    arrival_time: DateTime.utc_now(),
    destination_name: "",
    destination_stop: "",
    heading: "0",
    is_delayed: false,
    is_fault: false,
    is_scheduled: false,
    latitude: "",
    longitude: "",
    prediction_timestamp: DateTime.utc_now(),
    run: "",
    route: nil,
    station_id: "",
    station_name: "",
    stop_description: "",
    stop_id: "",
    train_direction: nil
  ]
  # @type t :: %__MODULE__{is_delayed: boolean(), destination: String.t(), heading: String.t(), latitude: String.t(), longitude: String.t(), prediction_timestamp: DateTime.t(), run: String.t(), route: String.t()}

  defp timestamp_str_to_datetime(str) do
    {:ok, timestamp, _offet} = DateTime.from_iso8601(str <> "-06")
    timestamp
  end

  def map_response_to_struct(vehicle) when is_map(vehicle) do
    %{
      "arrT" => arrival_time,
      "destNm" => destination_name,
      "destSt" => destination_stop,
      "isDly" => is_delayed,
      "isFlt" => is_fault,
      "isSch" => is_scheduled,
      "heading" => heading,
      "lat" => latitude,
      "lon" => longitude,
      "prdt" => prediction_timestamp,
      "rn" => run,
      "rt" => route,
      "staId" => station_id,
      "staNm" => station_name,
      "stpDe" => stop_description,
      "stpId" => stop_id,
      "trDr" => train_direction
    } = vehicle

    struct(__MODULE__,
      arrival_time: timestamp_str_to_datetime(arrival_time),
      destination_name: destination_name,
      destination_stop: destination_stop,
      heading: heading,
      is_delayed: is_delayed,
      is_fault: is_fault,
      is_scheduled: is_scheduled,
      latitude: latitude,
      longitude: longitude,
      prediction_timestamp: timestamp_str_to_datetime(prediction_timestamp),
      route: @train_routes[route],
      run: run,
      station_id: station_id,
      station_name: station_name,
      stop_description: stop_description,
      stop_id: stop_id,
      train_direction: if(train_direction == "1", do: :northbound, else: :southbound)
    )
  end

end
