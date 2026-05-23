import React from 'react'
import './MainNavigation.css'
import MainHeader from './MainHeader'
import { Link } from 'react-router-dom'
import NavigationLinks from './NavigationLinks'
import SideDrawer from './SideDrawer'
import Backdrop from '../UIElements/Backdrop'
import { useState } from 'react'

const MainNavigation = (props) => {
    const [drawerIsOpen, setDrawerIsOpen] = useState(false)
    const openDrawer = () => {
        setDrawerIsOpen(true)
    }
    const closeDrawer = () => {
        setDrawerIsOpen(false)
    }
    return (
        <React.Fragment>
            {drawerIsOpen &&
                <Backdrop onClick={closeDrawer} />}

            <SideDrawer show={drawerIsOpen} closeDrawer={closeDrawer}>
                <nav>
                    <NavigationLinks className="main-navigation__drawer-nav" />
                </nav>
            </SideDrawer>
            <MainHeader>
                <button className='main-navigation__menu-btn' onClick={openDrawer}>
                    {/* for hamburger icon */}
                    <span></span>
                    <span></span>
                    <span></span>
                </button>
                <h1 className='main-navigation__title'>
                    <Link to='/'>
                        INSTAPLACE
                    </Link>
                </h1>
                <nav className='main-navigation__header-nav'>
                    <NavigationLinks />
                </nav>
            </MainHeader>
        </React.Fragment>
    )
}

export default MainNavigation