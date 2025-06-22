import React from "react";
import { useAuth } from "../context/AuthContext";
import "bootstrap/dist/css/bootstrap.min.css";
import RegistrationForm from "../components/RegistrationForm";
import BoxList from "../components/BoxList";

const LoginPage = () => {
  const { isAuthenticated, principal, login, logout, loading, needsRegistration, userData, balance } = useAuth();

  if (loading) {
    return (
      <div className="d-flex justify-content-center my-4">
        <div className="spinner-border text-primary" role="status" />
      </div>
    );
  }

  return (
    <div className="card text-center shadow-sm p-4 mt-5 mx-auto">
      <div className="card-body">
        {!isAuthenticated ? (
          <button className="btn btn-primary w-100" onClick={login}>
            Login with Internet Identity
          </button>
        ) : needsRegistration ? (
          <>
            <p className="text-muted">Complete your registration</p>
            <RegistrationForm />
            <button className="btn btn-outline-danger w-100 mt-3" onClick={logout}>
              Cancel and Logout
            </button>
          </>
        ) : (
          <>
            <p className="card-text text-muted">Welcome, <strong>{userData.nickname}</strong></p>
            <p className="card-text text-muted">Your principal, <strong>{principal}</strong></p>
            <p className="card-text text-muted">Your balance: <strong>{balance} ICP</strong></p>
            <button className="btn btn-outline-danger w-100 mt-3" onClick={logout}>
              Logout
            </button>
          </>
        )}
      </div>
      <BoxList/>
    </div>
  );
};

export default LoginPage;