import "./App.css";
import McpClient from "./McpClient";

function App() {
  return (
    <main className="container">
      <h1>Tauri MCP Weather App</h1>
      <McpClient />
    </main>
  );
}

export default App;
