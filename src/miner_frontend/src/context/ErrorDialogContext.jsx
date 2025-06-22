import React, { createContext, useContext, useState } from 'react';

const ErrorDialogContext = createContext();
export const useErrorDialog = () => useContext(ErrorDialogContext);

export const ErrorDialogProvider = ({ children }) => {
  const [error, setError] = useState(null);
  const [visible, setVisible] = useState(false);

  const showError = (message) => {
    setError(message);
    setVisible(true);
  };

  const closeError = () => {
    setVisible(false);
    setTimeout(() => setError(null), 300); // unmount after fade-out
  };

  return (
    <ErrorDialogContext.Provider value={{ showError }}>
      {children}

      {/* Mount modal only when there's an error */}
      {error && (
        <>
          <div className={`modal fade ${visible ? 'show d-block' : ''}`} tabIndex="-1" role="dialog">
            <div className="modal-dialog modal-dialog-centered" role="document">
              <div className="modal-content">
                <div className="modal-header bg-danger text-white">
                  <h5 className="modal-title">Error</h5>
                  <button type="button" className="btn-close" onClick={closeError}></button>
                </div>
                <div className="modal-body">
                  <p>{error}</p>
                </div>
                <div className="modal-footer">
                  <button className="btn btn-secondary" onClick={closeError}>
                    Close
                  </button>
                </div>
              </div>
            </div>
          </div>

          {/* Backdrop only when modal is visible */}
          {visible && <div className="modal-backdrop fade show"></div>}
        </>
      )}
    </ErrorDialogContext.Provider>
  );
};
