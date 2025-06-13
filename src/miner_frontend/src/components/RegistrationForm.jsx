import React, { useState } from "react";
import { useAuth } from "../context/AuthContext";

const RegistrationForm = () => {
  const { register } = useAuth();
  const [nickname, setNickname] = useState("");
  const [message, setMessage] = useState(null);

  const handleSubmit = async (e) => {
    e.preventDefault();
    console.log("Start reg");
    const response = await register(nickname);
    console.log("setMessage", response);
    setMessage(response == true ? "Registration successful!" : response);
  };

  return (
    <form onSubmit={handleSubmit} className="mt-4">
      <div className="mb-3">
        <input
          className="form-control"
          type="text"
          placeholder="Nickname"
          value={nickname}
          onChange={(e) => setNickname(e.target.value)}
          required
        />
      </div>      
      <button type="submit" className="btn btn-success w-100">Register</button>
      {message && <div className="alert alert-info mt-3">{message}</div>}
    </form>
  );
};

export default RegistrationForm;