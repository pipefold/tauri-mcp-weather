import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./McpClient.css";

interface WeatherData {
  temperature: number;
  condition: string;
  humidity: number;
  windSpeed: number;
}

export default function McpClient() {
  const [cities, setCities] = useState<string[]>([]);
  const [selectedCity, setSelectedCity] = useState<string>("");
  const [weatherData, setWeatherData] = useState<WeatherData | null>(null);
  const [serverStatus, setServerStatus] = useState<string>("stopped");
  const [error, setError] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(false);

  // Start MCP server
  const startServer = async () => {
    try {
      setIsLoading(true);
      setError("");
      setServerStatus("starting");

      const result = await invoke("start_mcp_server");
      console.log(result);

      setServerStatus("running");
      // After starting server, fetch cities
      fetchCities();
    } catch (e) {
      console.error("Failed to start server:", e);
      setError(`Failed to start MCP server: ${e}`);
      setServerStatus("stopped");
    } finally {
      setIsLoading(false);
    }
  };

  // Stop MCP server
  const stopServer = async () => {
    try {
      setIsLoading(true);
      setError("");
      setServerStatus("stopping");

      const result = await invoke("stop_mcp_server");
      console.log(result);

      setServerStatus("stopped");
      setCities([]);
      setWeatherData(null);
    } catch (e) {
      console.error("Failed to stop server:", e);
      setError(`Failed to stop MCP server: ${e}`);
      // Keep the current server status
    } finally {
      setIsLoading(false);
    }
  };

  // Fetch list of cities from MCP server
  const fetchCities = async () => {
    if (serverStatus !== "running") {
      setError("Cannot fetch cities: MCP server not running");
      return;
    }

    try {
      setError("");
      // MCP message to get the cities resource
      const message = JSON.stringify({
        jsonrpc: "2.0",
        method: "get_resource",
        params: {
          uri: "weather://cities",
        },
        id: 1,
      });

      // Send message to MCP server
      const result = await invoke("send_to_mcp_server", { message });
      console.log("Server response:", result);

      // Parse the response (in a real app, you'd handle this properly)
      // This is simplified for the example
      setCities(["New York", "London", "Tokyo", "Sydney", "Paris"]);
    } catch (e) {
      console.error("Failed to fetch cities:", e);
      setError(`Failed to fetch cities: ${e}`);
    }
  };

  // Fetch weather data for a city using MCP tool
  const fetchWeatherData = async (city: string) => {
    if (!city) return;
    if (serverStatus !== "running") {
      setError("Cannot fetch weather data: MCP server not running");
      return;
    }

    try {
      setError("");
      // MCP message to invoke the getWeather tool
      const message = JSON.stringify({
        jsonrpc: "2.0",
        method: "invoke_tool",
        params: {
          tool: "getWeather",
          arguments: {
            city,
          },
        },
        id: 2,
      });

      // Send message to MCP server
      await invoke("send_to_mcp_server", { message });

      // In a real implementation, you'd parse the response from the server
      // This is simplified for the example
      const cityDataMap: Record<string, WeatherData> = {
        "New York": {
          temperature: 72,
          condition: "Sunny",
          humidity: 45,
          windSpeed: 8,
        },
        London: {
          temperature: 62,
          condition: "Rainy",
          humidity: 80,
          windSpeed: 12,
        },
        Tokyo: {
          temperature: 78,
          condition: "Partly Cloudy",
          humidity: 65,
          windSpeed: 5,
        },
        Sydney: {
          temperature: 85,
          condition: "Clear",
          humidity: 55,
          windSpeed: 10,
        },
        Paris: {
          temperature: 68,
          condition: "Cloudy",
          humidity: 70,
          windSpeed: 7,
        },
      };

      setWeatherData(cityDataMap[city]);
    } catch (e) {
      console.error("Failed to fetch weather data:", e);
      setError(`Failed to fetch weather data: ${e}`);
      setWeatherData(null);
    }
  };

  // Handle city selection
  const handleCityChange = (city: string) => {
    setSelectedCity(city);
    fetchWeatherData(city);
  };

  // Init effect
  useEffect(() => {
    // Don't auto-start on component mount
    // Let the user explicitly click Start Server
  }, []);

  // Get status text and style
  const getStatusDisplay = () => {
    switch (serverStatus) {
      case "running":
        return <span className="status-active">running</span>;
      case "starting":
        return <span className="status-pending">starting...</span>;
      case "stopping":
        return <span className="status-pending">stopping...</span>;
      default:
        return <span className="status-inactive">stopped</span>;
    }
  };

  return (
    <div className="mcp-client">
      <h2>MCP Weather Client</h2>

      <div className="server-controls">
        <p>Server Status: {getStatusDisplay()}</p>
        <div className="button-group">
          <button
            onClick={startServer}
            disabled={
              isLoading ||
              serverStatus === "running" ||
              serverStatus === "starting"
            }
            className="server-button start-button"
          >
            Start Server
          </button>
          <button
            onClick={stopServer}
            disabled={
              isLoading ||
              serverStatus === "stopped" ||
              serverStatus === "stopping"
            }
            className="server-button stop-button"
          >
            Stop Server
          </button>
        </div>
      </div>

      {error && <div className="error-message">{error}</div>}

      <div className="weather-section">
        <h3>City Weather</h3>
        <select
          value={selectedCity}
          onChange={(e) => handleCityChange(e.target.value)}
          disabled={cities.length === 0 || serverStatus !== "running"}
          className="city-select"
        >
          <option value="">Select a city</option>
          {cities.map((city) => (
            <option key={city} value={city}>
              {city}
            </option>
          ))}
        </select>

        {weatherData && (
          <div className="weather-info">
            <h4>{selectedCity}</h4>
            <div className="weather-details">
              <p>
                <strong>Temperature:</strong> {weatherData.temperature}°F
              </p>
              <p>
                <strong>Condition:</strong> {weatherData.condition}
              </p>
              <p>
                <strong>Humidity:</strong> {weatherData.humidity}%
              </p>
              <p>
                <strong>Wind Speed:</strong> {weatherData.windSpeed} mph
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
