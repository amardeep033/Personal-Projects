import React, { useEffect, useState,useContext } from 'react'
import { authContext } from '../../shared/Context/auth-context';
import { useHistory } from 'react-router-dom'
import { useParams } from 'react-router-dom'
import Input from '../../shared/components/FormElements/Input'
import Button from '../../shared/components/FormElements/Button'
import { VALIDATOR_MINLENGTH, VALIDATOR_REQUIRE } from '../../shared/util/validators'
import './PlaceForm.css'
import { useForm } from '../../shared/hooks/form-hook'
import { useHttp } from '../../shared/hooks/http-hook';
import LoadingSpinner from '../../shared/components/UIElements/LoadingSpinner';
import ErrorModal from '../../shared/components/UIElements/ErrorModal';

const UpdatePlace = () => {
    const  history=useHistory()
    const auth=useContext(authContext)
    const [isLoading, errMsg, sendReq, onclearHandler] = useHttp()
    const [idfPlace, setIdfPlace] = useState();
    const [formState, inputHandler, setFormData] = useForm({
        title: {
            value: '',
            isValid: false
        },
        description: {
            value: '',
            isValid: false
        }
    }, false)

    const placeId = useParams().placeId
    console.log(placeId)

    useEffect(() => {
        const fetchPlace = async () => {
            try {
                const respData = await sendReq(`http://localhost:5000/api/places/${placeId}`)
                console.log(respData)
                setIdfPlace(respData.place)
                setFormData({
                    title: {
                        value: respData.place.title,
                        isValid: true
                    },
                    description: {
                        value: respData.place.description,
                        isValid: true
                    }
                }, true)
            }
            catch (err) { }
        }
        fetchPlace()
    }, [sendReq, placeId, setFormData])

    const placeUpdateSubmitHandler = async(event) => {
        event.preventDefault();
        try {
            const respData = await sendReq(`http://localhost:5000/api/places/${placeId}`, 'PATCH', JSON.stringify({
              title: formState.inputs.title.value,
              description: formState.inputs.description.value
            }), { 'Content-Type': 'application/json' ,authorization:"Bearer "+auth.token})
            console.log(respData)
            history.push('/'+auth.userId+'/places')
          }
          catch (err) { }
    }

    return (
        <React.Fragment>
            <ErrorModal error={errMsg} onClear={onclearHandler}/>
            {isLoading && <LoadingSpinner asOverlay />}
            {!isLoading && idfPlace && 
            <form className='place-form' onSubmit={placeUpdateSubmitHandler}>
                <Input id='title'
                    element='input'
                    type='text'
                    label='title'
                    validators={[VALIDATOR_REQUIRE()]}
                    errorText="Please enter a valid title"
                    onInput={inputHandler}
                    initialValue={idfPlace.title}
                    initialValid={true}></Input>

                <Input id='description'
                    element='textarea'
                    label='description'
                    validators={[VALIDATOR_MINLENGTH(5)]}
                    errorText="Please enter atleast 5 character"
                    onInput={inputHandler}
                    initialValue={idfPlace.description}
                    initialValid={true}></Input>

                <Button type='submit' disabled={!formState.isValid}>Update Place</Button>
            </form>}
        </React.Fragment>
    )
}

export default UpdatePlace