import React,{useState} from 'react'
import './PlaceItem.css'
import Card from '../../shared/components/UIElements/Card'
import Button from '../../shared/components/FormElements/Button'
import { authContext } from '../../shared/Context/auth-context'
import { useContext } from 'react'
import Modal from '../../shared/components/UIElements/Modal'
import { useHttp } from '../../shared/hooks/http-hook';
import ErrorModal from '../../shared/components/UIElements/ErrorModal'
import LoadingSpinner from '../../shared/components/UIElements/LoadingSpinner'

const PlaceItem = (props) => {
  const [isLoading, errMsg, sendReq, onclearHandler] = useHttp()

  const [showMap, setShowMap] = useState(false);
  const openMapHandler = () => setShowMap(true);
  const closeMapHandler = () => setShowMap(false);

  const [showDel, setShowDel] = useState(false);
  const openDelHandler = () => setShowDel(true);
  const closeDelHandler = () => setShowDel(false);
  const submitDelHandler=async()=>{
    // console.log("deleting...")
    try {
      props.onDeleteHandler(props.id)
      const respData = await sendReq(`http://localhost:5000/api/places/${props.id}`, 'DELETE',null,{authorization:"Bearer "+auth.token})
      console.log(respData)
      closeDelHandler()
    }
    catch (err) { }
  }

  const auth=useContext(authContext)
  return (
    <React.Fragment>
    <ErrorModal error={errMsg} onClear={onclearHandler}/>

    <Modal
        show={showMap}
        onCancel={closeMapHandler}
        header={props.address}
        contentClass="place-item__modal-content"
        footerClass="place-item__modal-actions"
        footer={<Button onClick={closeMapHandler}>CLOSE</Button>}
      >
        <div className="map-container">
          <h2>THE MAP!</h2>
        </div>
      </Modal>

      <Modal
        show={showDel}
        onCancel={closeDelHandler}
        header="CONFIRM DELETION WARNING"
        footerClass="place-item__modal-actions"
        footer={<React.Fragment>
            <Button inverse onClick={closeDelHandler}>Cancel</Button>
            <Button danger onClick={submitDelHandler}>Delete</Button>
        </React.Fragment>}>
        <div><p>Do you really want to delete the place permanently? Note: The process is non recoverable.</p></div>
      </Modal>

    <li className='place-item'>
      <Card className='place-item__content'>
        {isLoading && <LoadingSpinner asOverlay />}
        <div className='place-item__image'>
          <img src={`http://localhost:5000/${props.image}`} alt={props.title} />
        </div>
        <div className='place-item__info'>
          <h2>{props.title}</h2>
          <h3>{props.address}</h3>
          <p>{props.description}</p>
        </div>
        <div className='place-item__actions'>
          <Button inverse onClick={openMapHandler}>VIEW ON MAP</Button>
          {auth.isLoggedIn && auth.userId===props.creatorId && <Button to={`/places/${props.id}`}>EDIT</Button>}
          {auth.isLoggedIn && auth.userId===props.creatorId && <Button danger onClick={openDelHandler}>DELETE</Button>}
        </div>
      </Card>
    </li>
    </React.Fragment>
  )
}

export default PlaceItem