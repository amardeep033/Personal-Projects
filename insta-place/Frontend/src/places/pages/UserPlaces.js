import React,{useState,useEffect} from 'react'
import PlaceList from '../components/PlaceList'
import { useParams } from 'react-router-dom'
import LoadingSpinner from '../../shared/components/UIElements/LoadingSpinner';
import ErrorModal from '../../shared/components/UIElements/ErrorModal';
import { useHttp } from '../../shared/hooks/http-hook';

const UserPlaces = () => {

    const [isLoading, errMsg, sendReq, onclearHandler] = useHttp()
    const [loadedUserPlaces, setLoadedUserPlaces] = useState();
    const userId=useParams().userId
    // const loadedPlaces=PLACES.filter(place=>place.creatorId===userId)
    useEffect(()=>{
        const fetchUserPlaces=async()=>{
          try{
            const respData=await sendReq(`http://localhost:5000/api/places/user/${userId}`)
            console.log(respData)
            setLoadedUserPlaces(respData.places)
          }
          catch(err) {}
        }
        fetchUserPlaces()
    },[sendReq,userId])
    const onDeleteHandler=(delPlaceId)=>{
      setLoadedUserPlaces(prevLoadedUserPlaces=>prevLoadedUserPlaces.filter(p=>p.id!==delPlaceId))
    }
    return (
        <React.Fragment>
            <ErrorModal error={errMsg} onClear={onclearHandler}/>
            {isLoading && <LoadingSpinner asOverlay />}
            {!isLoading && loadedUserPlaces && <PlaceList items={loadedUserPlaces} onDeleteHandler={onDeleteHandler} />}
        </React.Fragment>
    )
}

export default UserPlaces