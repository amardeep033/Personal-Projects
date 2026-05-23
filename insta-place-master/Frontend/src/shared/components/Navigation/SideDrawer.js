import React from 'react'
import ReactDOM from 'react-dom';
import { CSSTransition } from 'react-transition-group';

import './SideDrawer.css'

const SideDrawer = props => {
  const stored = <CSSTransition
    in={props.show}
    timeout={200}
    classNames='slide-in-left'
    mountOnEnter unmountOnExit>
    <aside className='side-drawer' onClick={props.closeDrawer}>{props.children}</aside>
  </CSSTransition>
  return ReactDOM.createPortal(stored,
    document.getElementById('drawer-hook')
  );
}

export default SideDrawer