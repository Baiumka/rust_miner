import React from "react";
import { AuthProvider } from "./context/AuthContext";
import LoginPage from "./pages/LoginPage";
import { ErrorDialogProvider } from './context/ErrorDialogContext';
import { PromptDialogProvider } from './context/PromptDialogContext';

function App() {
  return (
    <PromptDialogProvider>
    <ErrorDialogProvider>
      <AuthProvider>      
        <div className="App">        
          <LoginPage />
        </div>
      </AuthProvider>
    </ErrorDialogProvider>
    </PromptDialogProvider>
  );
}

export default App;