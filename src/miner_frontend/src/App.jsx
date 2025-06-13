import React from "react";
import { AuthProvider } from "./context/AuthContext";
import LoginPage from "./pages/LoginPage";

function App() {
  return (
    <AuthProvider>
      <div className="App">        
        <LoginPage />
      </div>
    </AuthProvider>
  );
}

export default App;