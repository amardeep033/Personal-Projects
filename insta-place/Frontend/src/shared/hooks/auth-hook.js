import { useState ,useCallback,useEffect} from "react"

export const useAuth=()=>{
  const[isLoggedIn,setIsLoggedIn]=useState(false)
  const[userId,setUserId]=useState(null)
  const[token,setToken]=useState(null)

  const login=useCallback((uid,token,exp)=>{
    setToken(token) //token for a&a
    setUserId(uid)
    setIsLoggedIn(true)
    const expTime=exp || new Date(new Date().getTime() + 1000*60*60) //for exp
    localStorage.setItem('userData',JSON.stringify({userId:uid,token:token,expTime:expTime.toISOString()}))  //for refresh
  },[])
  
  const logout=useCallback(()=>{
    setToken(null)
    setUserId(null)
    setIsLoggedIn(false)
    localStorage.removeItem('userData') //for new user.. new token
  },[])

  //login back on refresh...
  useEffect(() => {
    const storedData=JSON.parse(localStorage.getItem('userData'))
    if(storedData && storedData.token && new Date(storedData.expTime)>new Date()) //not expired call login
    {
      login(storedData.userId,storedData.token,new Date(storedData.expTime)) //send exptime too
    }
  }, [login])

  return [login,logout,isLoggedIn,userId,token]
}