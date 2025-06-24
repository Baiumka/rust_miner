import React from "react";
import { AuthProvider } from "./context/AuthContext";
import LoginPage from "./pages/LoginPage";
import { ErrorDialogProvider } from './context/ErrorDialogContext';
import { PromptDialogProvider } from './context/PromptDialogContext';
import { IdentityKitProvider } from "@nfid/identitykit/react"

function App() {
  return (
    <IdentityKitProvider>
    <PromptDialogProvider>
    <ErrorDialogProvider>
      <AuthProvider>      
        <div className="App">        
          <LoginPage />
        </div>
      </AuthProvider>
    </ErrorDialogProvider>
    </PromptDialogProvider>
    </IdentityKitProvider>
  );
}

export default App;