import { useCallback, useState} from 'react';

export const useHttp = () => {
    const [isLoading, setIsLoading] = useState(false);
    const [errMsg, setErrMsg]=useState()

    const sendReq=useCallback(async(url,method='GET',body=null,headers={})=>{
        try{
            setIsLoading(true)
            const resp=await fetch(url,{
            method,
            headers,
            body})
            const respData=await resp.json()
            if(!resp.ok)
            {
            throw new Error(respData.message)
            }
            setIsLoading(false)
            return respData
        }
        catch(err)
        {
          setIsLoading(false)
          setErrMsg(err.message || 'Something went wrong') //from backend when response not okayish
          throw err 
        }
    },[])

    const onclearHandler=()=>{
        setErrMsg(null)
    }

  return [isLoading,errMsg,sendReq,onclearHandler];
};