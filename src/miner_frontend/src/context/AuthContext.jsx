import { IDL } from "@dfinity/candid";
import { Principal } from '@dfinity/principal';
import React, { createContext, useContext, useEffect, useState } from "react";
import { AuthClient } from "@dfinity/auth-client";
import { createActor, canisterId, idlFactory } from 'declarations/miner_backend';
import { miner_backend } from "../../../declarations/miner_backend"
import { IcrcLedgerCanister } from "@dfinity/ledger-icrc";
import { createAgent } from "@dfinity/utils";

const AuthContext = createContext();

export const AuthProvider = ({ children }) => {

  const [identity, setIdentity] = useState(null);
  const [principal, setPrincipal] = useState(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [userData, setUserData] = useState(null);
  const [needsRegistration, setNeedsRegistration] = useState(false);
  const [loading, setLoading] = useState(true);
  const [userActor, setUserActor] = useState(null);
  const [balance, setBalance] = useState(null);

  useEffect(() => {
    initAuth();
  }, []);

  const initAuth = async () => {
    const client = await AuthClient.create();    
    //client.logout();
    if (await client.isAuthenticated()) {
      const identity = client.getIdentity();
      fillUserData(identity);
    }    
    else
    {
        setLoading(false);
    }
  };

  const getIdentityProvider = () => {
    let idpProvider;
    if (typeof window !== "undefined") {
      const isLocal = process.env.DFX_NETWORK == "local";    
      const isSafari = /^((?!chrome|android).)*safari/i.test(navigator.userAgent);
      if (isLocal && isSafari) {
        idpProvider = `http://localhost:4943/?canisterId=${process.env.CANISTER_ID_INTERNET_IDENTITY}`;
      } else if (isLocal) {
        idpProvider = `http://${process.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:4943`;
      }
      else
      {
        idpProvider = `https://identity.ic0.app/#authorize`;
      }
    }    
    return idpProvider;
  };

  const login = async () => {
    const client = await AuthClient.create();
    await client.login({
      identityProvider: getIdentityProvider(),
      onSuccess: async () => {
        const identity = client.getIdentity();
        fillUserData(identity);        
      },
    });
  };


  const getAllBoxes = async () => {
    let boxes;
    if(userActor)
    {
      boxes = await userActor.get_all_boxes();      
    }
    else
    {
      boxes = await miner_backend.get_all_boxes();
    }
    console.log(boxes);
    return boxes;
  };

  const fillUserData = async (userIdentity) => {
    const principal = userIdentity.getPrincipal().toString();
    const actor = createActor(canisterId, { agentOptions: { identity: userIdentity } });
    const response = await actor.get_user();        
    if ("Ok" in response) {
        setUserData(response.Ok);
        setNeedsRegistration(false);
        const balance = await actor.get_my_balance();
        console.log("balance",balance);
        if(balance.Ok)
        {          
          setBalance(Number(balance.Ok) / 100_000_000);
        }
        else
        {
          setBalance(0);
        }
    }
    else
    {
        setUserData(null);
        setNeedsRegistration(true);
    }
    setLoading(false);
    setIdentity(userIdentity);
    setPrincipal(principal);
    setIsAuthenticated(true);        
    setUserActor(actor);
  };

  const logout = async () => {
    const client = await AuthClient.create();
    await client.logout();
    setIsAuthenticated(false);
    setPrincipal(null);
    setUserData(null);
    setNeedsRegistration(false);
  };

  const register = async (nickname) => {          
    const response = await userActor.register(nickname);
    if ("Ok" in response) {
        setUserData(response.Ok);
        setNeedsRegistration(false);
        return true;
    }
    else
    {
        setUserData(null);
        setNeedsRegistration(true);
        return response.Err;
    }
  };

  const approve = async (icp) => {   
    const provider =  getIdentityProvider();    
    console.log("provider",provider);
    const agent = await createAgent({
      identity,
      host: provider
    });

    if (process.env.DFX_NETWORK == "local") {
      agent.fetchRootKey().catch((err) => {
        console.warn(
          "Unable to fetch root key. Check to ensure that your local replica is running"
        );
        console.error(err);
      });
    }
  
    const ledger = IcrcLedgerCanister.create({
      agent,
      canisterId: "ryjl3-tyaaa-aaaaa-aaaba-cai",
    });    
    try
    {
      const result = await ledger.approve({
      // fee: null,
      // memo: null,
      // from_subaccount: [],
      // created_at_time: null,
        spender: { owner: Principal.fromText(canisterId), subaccount: [] },
        amount: icp + 10_000,
      // expected_allowance: null,
      // expires_at: null
      });          
      return { Ok: result};
    }
    catch (e)
    {
      return { Err: e.message};
    }
    

  };

  const createBox = async (icp) => {   
    const icp64 = icp * 100_000_000;
    const approve_result = await approve(icp64);        
    if(approve_result.Ok)
    {
      const response = await userActor.create_box(icp64);
      const balance = await userActor.get_my_balance();      
      if(balance.Ok)
      {          
        setBalance(Number(balance.Ok) / 100_000_000);
      }
      else
      {
        setBalance(0);
      }      
      return response;    
    }
    else
    {      
      return approve_result;
    }    
  };

  const useBox = async (box, icp) => {   
    const icp64 = icp * 100_000_000;
    const approve_result = await approve(icp64);    
    if(approve_result)
    {
      const response = await userActor.create_miner(box.canister_id, icp64);
      const balance = await userActor.get_my_balance();      
      if(balance.Ok)
      {          
        setBalance(Number(balance.Ok) / 100_000_000);
      }
      else
      {
        setBalance(0);
      }            
      return response;    
    }
    else
    {      
      return approve_result;
    }    
  };


  return (
    <AuthContext.Provider
      value={{
        isAuthenticated,
        principal,
        login,
        logout,
        loading,
        userData,
        needsRegistration,
        register,
        getAllBoxes,
        createBox,
        balance,
        useBox
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => useContext(AuthContext);