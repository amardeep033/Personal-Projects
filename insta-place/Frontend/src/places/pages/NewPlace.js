import React, { useContext } from 'react'
import { useHistory } from 'react-router-dom'
import './PlaceForm.css'
import Input from '../../shared/components/FormElements/Input'
import { VALIDATOR_MINLENGTH, VALIDATOR_REQUIRE } from '../../shared/util/validators'
import Button from '../../shared/components/FormElements/Button'
import { useForm } from '../../shared/hooks/form-hook'
import { useHttp } from '../../shared/hooks/http-hook'
import LoadingSpinner from '../../shared/components/UIElements/LoadingSpinner'
import ErrorModal from '../../shared/components/UIElements/ErrorModal'
import { authContext } from '../../shared/Context/auth-context'
import ImageUpload from '../../shared/components/FormElements/ImageUpload';


const NewPlace = () => {
  const auth = useContext(authContext)
  const [isLoading, errMsg, sendReq, onclearHandler] = useHttp()
  const [formState, inputHandler] = useForm({
    title: {
      value: '',
      isValid: false
    },
    description: {
      value: '',
      isValid: false
    },
    address: {
      value: '',
      isValid: false
    },
    image: {
      value: null,
      isValid: false
    }
  }, false)
  const history = useHistory()
  const placeSubmitHandler = async (event) => {
    event.preventDefault();
    console.log(formState.inputs)
    try {
      const formData = new FormData()
      formData.append('title', formState.inputs.title.value)
      formData.append('description', formState.inputs.description.value)
      formData.append('address', formState.inputs.address.value)
      formData.append('image', formState.inputs.image.value)
      const respData = await sendReq('http://localhost:5000/api/places/', 'POST', formData,{authorization:"Bearer "+auth.token})
      console.log(respData)
      history.push('/')
    }
    catch (err) { }
  }

  return (
    <React.Fragment>
      <ErrorModal error={errMsg} onClear={onclearHandler} />
      <form className='place-form' onSubmit={placeSubmitHandler}>
        {isLoading && <LoadingSpinner asOverlay />}
        <Input element='input'
          type='text'
          label='title'
          id='title'
          validators={[VALIDATOR_REQUIRE()]}
          onInput={inputHandler}
          errorText='Please enter a valid title'></Input>

        <ImageUpload center
          onInput={inputHandler}
          id="image"
        />

        <Input element='input'
          type='text'
          label='address'
          id='address'
          validators={[VALIDATOR_REQUIRE()]}
          onInput={inputHandler}
          errorText='Please enter a valid address'></Input>

        <Input element='textarea'
          id='description'
          label='description'
          validators={[VALIDATOR_MINLENGTH(5)]}
          onInput={inputHandler}
          errorText='Please enter atleast 5 character'></Input>

        <Button type='submit' disabled={!formState.isValid}>Add Place</Button>
      </form>
    </React.Fragment>
  )
}

export default NewPlace