import React,{useState,useEffect} from 'react'
import UsersList from '../components/UsersList'
import LoadingSpinner from '../../shared/components/UIElements/LoadingSpinner'
import ErrorModal from '../../shared/components/UIElements/ErrorModal'
import { useHttp } from '../../shared/hooks/http-hook'

const Users = () => {
  const [loadedUsers, setLoadedUsers] = useState();
  const [isLoading, errMsg, sendReq, onclearHandler] = useHttp()
  useEffect(()=>{
      const fetchUsers=async()=>{
        try{
          const respData=await sendReq('http://localhost:5000/api/users')
          console.log(respData)
          setLoadedUsers(respData.users)
        }
        catch(err) {}
      }
      fetchUsers()
  },[sendReq])
  return (
    <React.Fragment>
      <ErrorModal error={errMsg} onClear={onclearHandler}/>
      {isLoading && <LoadingSpinner asOverlay/>}
      {!isLoading && loadedUsers && <UsersList items={loadedUsers} />}
    </React.Fragment>
  )
}

export default Users