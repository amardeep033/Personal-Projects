import React, { useRef, useState, useEffect } from "react";
import Button from "./Button";
import './ImageUpload.css'

const ImageUpload = (props) => {
    const filePickerRef=useRef()
    
    const [file, setFile] = useState();
    const [previewUrl, setPreviewUrl] = useState();
    const [isValid, setIsValid] = useState(false);

    const pickImageHandler=()=>{
        filePickerRef.current.click()
    }

    useEffect(() => {
        if (!file) {
          return;
        }
        const fileReader = new FileReader();
        fileReader.onload = () => {
          setPreviewUrl(fileReader.result);
        };
        fileReader.readAsDataURL(file);
      }, [file]);

    const pickedImageHandler=(event)=>{
        // console.log(event.target)
        let pickedFile;
        let fileIsValid = isValid;
        if (event.target.files && event.target.files.length === 1) 
        {
            pickedFile = event.target.files[0];
            setFile(pickedFile);
            setIsValid(true);
            fileIsValid = true;
        } 
        else 
        {
            setIsValid(false);
            fileIsValid = false;
        }
        props.onInput(props.id, pickedFile, fileIsValid);
    }

    return (
    <div className="form-control">
        <input type="file" id={props.id} style={{ display: "none" }} accept=".jpg,.png,.jpeg" 
        ref={filePickerRef}
        onChange={pickedImageHandler}/>
        <div className={`image-upload ${props.center && 'center'}`}>
            <div className="image-upload__preview">
                {previewUrl && <img src={previewUrl} alt="preview"/>}
                {!previewUrl && <p>Please pick an image!</p>}
            </div>
            <Button type='button' onClick={pickImageHandler}>PICK IMAGE</Button>
        </div>
    </div>)
}

export default ImageUpload