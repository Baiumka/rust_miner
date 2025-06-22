import React, { createContext, useContext, useEffect, useState } from "react";
import { useAuth } from "../context/AuthContext";
import "bootstrap/dist/css/bootstrap.min.css";
import BoxCard from "../components/BoxCard";
import { useErrorDialog } from '../context/ErrorDialogContext';
import { usePromptDialog } from '../context/PromptDialogContext';


const BoxList = () => {

    const { getAllBoxes, createBox, isAuthenticated, needsRegistration} = useAuth();
    const [boxes, setBoxes] = useState([]);    
    const { showError } = useErrorDialog();
    const { showPrompt } = usePromptDialog();

    useEffect(() => {
        loadBoxes();
    }, []);

    const loadBoxes = async () => {
        const response = await getAllBoxes();
        setBoxes(response);
    };

    const addBox = async () => {
        if(isAuthenticated && !needsRegistration)
        {
            const result = await showPrompt("How much ICP will be as prize? (min 5.000 ICP):", "5.0000");
            if (result !== null) {
                const response = await createBox(result);
                if(response.Ok)
                {                                    
                    setBoxes(prev => [response.Ok, ...prev]);
                }
                else
                {
                    showError(response.Err);
                }
            } 
        }
        else
        {
            showError('You need to log in.');
        }
    };

    return (
        <div className="container mt-4">
          <div className="row">
            <div className="col-md-4 mb-4">
                    <BoxCard box={null} addBox={addBox}/>
            </div>
            {boxes.map((box, index) => {              
              return (
                <div className="col-md-4 mb-4" key={index}>
                    <BoxCard box={box} />
                </div>
              );
            })}
          </div>
        </div>
      );
};

export default BoxList;