import React, { createContext, useContext, useState } from 'react';

const PromptDialogContext = createContext();
export const usePromptDialog = () => useContext(PromptDialogContext);

export const PromptDialogProvider = ({ children }) => {
  const [visible, setVisible] = useState(false);
  const [message, setMessage] = useState('');
  const [inputValue, setInputValue] = useState('');
  const [resolvePromise, setResolvePromise] = useState(null);

  const showPrompt = (msg, defaultValue = '') => {
    setMessage(msg);
    setInputValue(defaultValue);
    setVisible(true);
    return new Promise((resolve) => {
      setResolvePromise(() => resolve);
    });
  };

  const handleConfirm = () => {
    setVisible(false);
    if (resolvePromise) resolvePromise(inputValue);
  };

  const handleCancel = () => {
    setVisible(false);
    if (resolvePromise) resolvePromise(null);
  };

  return (
    <PromptDialogContext.Provider value={{ showPrompt }}>
      {children}

      {visible && (
        <>
          <div className="modal fade show d-block" tabIndex="-1" role="dialog">
            <div className="modal-dialog modal-dialog-centered" role="document">
              <div className="modal-content">
                <div className="modal-header">
                  <h5 className="modal-title">Input Required</h5>
                  <button type="button" className="btn-close" onClick={handleCancel}></button>
                </div>
                <div className="modal-body">
                  <p>{message}</p>
                  <input
                    type="number"
                    className="form-control"
                    step="0.0001"
                    value={inputValue}
                    onChange={(e) => setInputValue(e.target.value)}
                    min="5"
                    autoFocus
                  />
                </div>
                <div className="modal-footer">
                  <button className="btn btn-secondary" onClick={handleCancel}>Cancel</button>
                  <button className="btn btn-primary" onClick={handleConfirm}>OK</button>
                </div>
              </div>
            </div>
          </div>
          <div className="modal-backdrop fade show"></div>
        </>
      )}
    </PromptDialogContext.Provider>
  );
};
