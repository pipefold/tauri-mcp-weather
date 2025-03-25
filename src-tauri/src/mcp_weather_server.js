import { McpServer, ResourceTemplate } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/transport/stdio.js";
import { z } from "zod";

// Sample weather data
const weatherData = {
  "New York": {
    temperature: 72,
    condition: "Sunny",
    humidity: 45,
    windSpeed: 8
  },
  "London": {
    temperature: 62,
    condition: "Rainy",
    humidity: 80,
    windSpeed: 12
  },
  "Tokyo": {
    temperature: 78,
    condition: "Partly Cloudy",
    humidity: 65,
    windSpeed: 5
  },
  "Sydney": {
    temperature: 85,
    condition: "Clear",
    humidity: 55,
    windSpeed: 10
  },
  "Paris": {
    temperature: 68,
    condition: "Cloudy",
    humidity: 70,
    windSpeed: 7
  }
};

// Create the MCP server
const server = new McpServer({
  name: "Weather MCP Server",
  version: "1.0.0",
  description: "A server that provides weather data via MCP"
});

// Add a resource to get all available cities
server.resource(
  "cities",
  "weather://cities",
  async (uri) => ({
    contents: [{
      uri: uri.href,
      text: JSON.stringify(Object.keys(weatherData))
    }]
  })
);

// Add a resource to get weather data for a specific city
server.resource(
  "cityWeather",
  new ResourceTemplate("weather://city/{cityName}", { list: undefined }),
  async (uri, { cityName }) => {
    const city = decodeURIComponent(cityName);
    const data = weatherData[city];
    
    if (!data) {
      return {
        contents: [{
          uri: uri.href,
          text: `No weather data available for ${city}`
        }]
      };
    }
    
    return {
      contents: [{
        uri: uri.href,
        text: JSON.stringify(data, null, 2)
      }]
    };
  }
);

// Add a tool to get weather for a city
server.tool(
  "getWeather",
  { city: z.string() },
  async ({ city }) => {
    const data = weatherData[city];
    
    if (!data) {
      return {
        content: [{ 
          type: "text", 
          text: `No weather data available for ${city}` 
        }],
        isError: true
      };
    }
    
    return {
      content: [{ 
        type: "text", 
        text: `Weather in ${city}:\nTemperature: ${data.temperature}°F\nCondition: ${data.condition}\nHumidity: ${data.humidity}%\nWind Speed: ${data.windSpeed} mph` 
      }]
    };
  }
);

// Add a tool to convert temperature
server.tool(
  "convertTemperature",
  { 
    temperature: z.number(),
    from: z.enum(["celsius", "fahrenheit"]),
    to: z.enum(["celsius", "fahrenheit"])
  },
  async ({ temperature, from, to }) => {
    if (from === to) {
      return {
        content: [{ type: "text", text: `${temperature}` }]
      };
    }
    
    let result;
    if (from === "celsius" && to === "fahrenheit") {
      result = (temperature * 9/5) + 32;
    } else {
      result = (temperature - 32) * 5/9;
    }
    
    return {
      content: [{ 
        type: "text", 
        text: `${result.toFixed(1)}` 
      }]
    };
  }
);

// Define a prompt for getting weather information
server.prompt(
  "weatherPrompt",
  { city: z.string() },
  ({ city }) => ({
    messages: [{
      role: "user",
      content: [{ 
        type: "text", 
        text: `What's the weather like in ${city}?` 
      }]
    }]
  })
);

// Start the server using stdio
const transport = new StdioServerTransport();
await server.connect(transport);

console.error("MCP Weather Server started and waiting for input..."); 