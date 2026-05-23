import React from 'react';
import ReactDOM from 'react-dom';

import './Backdrop.css';

const Backdrop = props => {
  const stored=<div className="backdrop" onClick={props.onClick}></div>
  return ReactDOM.createPortal(stored,
    document.getElementById('backdrop-hook')
  );
};

export default Backdrop;
