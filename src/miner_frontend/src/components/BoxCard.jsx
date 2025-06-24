import React, { useState, useEffect } from "react";

const BoxCard = ({ box, addBox, useBox }) => {
  const [isHovered, setIsHovered] = useState(false);
  const [newBoxLoading, setNewBoxLoading] = useState(false);

  const Countdown = ({ endDateNano }) => {
    const [timeLeft, setTimeLeft] = useState(getTimeLeft(endDateNano));

    useEffect(() => {
        const interval = setInterval(() => {
            setTimeLeft(getTimeLeft(endDateNano));
        }, 1000);

        return () => clearInterval(interval); 
    }, [endDateNano]);

    return <span>{timeLeft}</span>;
    };

    function getTimeLeft(endDateNano) {
        const now = Date.now();
        const end = Number(endDateNano) / 1_000_000; 
        const diff = end - now;

        if (diff <= 0) return "Active";

        const seconds = Math.floor(diff / 1000) % 60;
        const minutes = Math.floor(diff / (1000 * 60)) % 60;
        const hours = Math.floor(diff / (1000 * 60 * 60)) % 24;
        const days = Math.floor(diff / (1000 * 60 * 60 * 24));

        return `${days}d ${hours}h ${minutes}m ${seconds}s`;
    }

    const addBoxHandler = async () => {
        setNewBoxLoading(true);
        const response = await addBox();
        setNewBoxLoading(false);
    };

    const useBoxHandler = async () => {
        setNewBoxLoading(true);
        const response = await useBox(box);
        setNewBoxLoading(false);
    };

  return (
    
        <div className="card hover-card h-100 shadow-sm" onMouseEnter={() => setIsHovered(true)} onMouseLeave={() => setIsHovered(false)}>    
        { box ? (      
            <div>            
                <img                    
                    src={isHovered ? "chest.png" : "closed_chest.png"}
                    className="card-img-top first"                            
                />                                    
                <div className="card-body">
                    {!newBoxLoading ? (
                        <>
                            {isHovered ? <button className="btn btn-success w-100" onClick={useBoxHandler}>Take it</button>: <></>}
                        </>
                    ) : (
                        <div className="d-flex justify-content-center my-4">
                            <div className="spinner-border text-primary" role="status" />
                        </div>    
                    )}                                               
                    <strong>Creator:</strong> {box.username}
                    <p className="card-text">                              
                    <strong>Time Left:</strong> <Countdown endDateNano={box.end_date} /> <br />      
                    <strong>Miner Count:</strong> {box.miner_count}
                    </p>           
                    {box.user_miners && box.user_miners.length > 0 ? (
                        <>
                        <strong>Your Miners:</strong>
                            {box.user_miners.map((miner, index) => {              
                               return (
                                <div className="d-flex align-items-center gap-2 small" key={index}>
                                  <div className="fs-6 fw-light">
                                    <strong>ID:</strong> {miner.canister_id}
                                  </div>
                                  <strong><Countdown endDateNano={miner.end_date} /></strong>
                                </div>
                              );
                            })}   
                        </>      
                    ) : (
                        <></>
                    )}
                </div>
            </div>
        ) : ( 
            <div>            
                <div>            
                    <img
                        src="empty_chest.png"
                        className="card-img-top first"                            
                    />                    
                    <div className="card-body">
                        {!newBoxLoading ? (
                            <button className="btn btn-primary w-100" onClick={addBoxHandler}>Creater new box</button>  
                        ) : (
                            <div className="d-flex justify-content-center my-4">
                                <div className="spinner-border text-primary" role="status" />
                            </div>    
                        )}                                          
                    </div>
                </div>
            </div>
        )}
        </div>    
  );
};

export default BoxCard;
