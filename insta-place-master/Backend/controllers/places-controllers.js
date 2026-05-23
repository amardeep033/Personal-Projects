const HttpError=require('../models/http-error')
const mongoose=require('mongoose')
const User=require('../models/USER')
const fs=require('fs')

const {validationResult}=require('express-validator')

const Place=require('../models/PLACE')

const getPlaceByPid=async(req,res,next)=>{
        const placeId=req.params.pid
        //find
        try{
            const place=await Place.findById(placeId)
            if(!place)
                // return res.status(404).json({message:'Not available'})
                return next(new HttpError('Not available',404))
            console.log('Get places')
            // res.status(200).json({place})
            res.status(200).json({place:place.toObject({getters:true})})
        }
        catch(err)
        {
            return next(new HttpError(err.message,500))
        }
}

const getPlacesByUid=async(req,res,next)=>{
    const userId=req.params.uid
    //filter
    try{
        const places=await Place.find({creatorId:userId})
        // const places = await User.findById(userId).populate('places');
        console.log(places)
        if(!places || places.length===0)
            return next(new HttpError('No places for this user',404))
        console.log('Get users places')
        res.status(200).json({places:places.map(p=>p.toObject({getters:true}))})
    }
    catch(err)
    {
        // res.status(500).json({message:err.message})
        return next(new HttpError(err.message,500))
    }
}

const createPlace=async(req,res,next)=>{
    // const title=req.body.title
    const valRes=validationResult(req)
    if(!valRes.isEmpty())
    {
        console.log(valRes)
        return next(new HttpError('empty or invalid inputs',422))
    }
    const {title,description,address}=req.body
    const  creatorId=req.userData.userId
    let user
    try{
        user=await User.findById(creatorId)
    }
    catch(err)
    {
        // res.status(500).json({message:err.message})
        return next(new HttpError(err.message,500))
    }
    if(!user)
    {
        console.log('user not found for creation')
        return next(new HttpError('user not found',404))
    }
    console.log(user)
    let createdPlace
    try{
        createdPlace=new Place({
            title,description,address,creatorId,image:req.file.path
        })
        const sess=await mongoose.startSession()
        sess.startTransaction()
        await createdPlace.save({session:sess})
        user.places.push(createdPlace)
        await user.save({session:sess})
        await sess.commitTransaction()
        console.log('place created')
    }
    catch(err)
    {
        console.log('place not created')
        // res.status(500).json({message:err.message})
        return next(new HttpError(err.message,500))
    }
    res.status(200).json({createdPlace:createdPlace.toObject({getters:true})})
}

const updatePlace=async(req,res,next)=>{
    const valRes=validationResult(req)
    if(!valRes.isEmpty())
    {
        console.log(valRes)
        return next(new HttpError('empty or invalid inputs',422))
    }
    const placeId=req.params.pid
    const {title,description}=req.body
    // const place=PLACES.find(p=>{
    //     return p.id===placeId})
    try{
        const place=await Place.findById(placeId)
        if(place.creatorId.toString() !== req.userData.userId)
        {
            throw new Error('You are not allowed to update this place');
        }
        place.title=title
        place.description=description
        await place.save()
        console.log('Place updated')
        res.status(200).json({updatedPlace:place.toObject({getters:true})})
    }
    catch(err)
    {
        // res.status(500).json({message:err.message})
        return next(new HttpError(err.message,500))
    }
}

const deletePlace=async(req,res,next)=>{
    const placeId=req.params.pid
    try{
        const place=await Place.findById(placeId)
        // const place=await Place.findById(placeId).populate('creatorId')...if two-way...user=>place.creator
        if(!place)
        {
            console.log('No such place found to delete')
            // res.status(200).json({message:'Place Deleted'})
            return next(new HttpError('No such place found to delete',404))
        }
        const creatorId=place.creatorId.toString()
        if(creatorId !== req.userData.userId)
        {
            throw new Error('You are not allowed to delete this place');
        }
        const user=await User.findById(creatorId)
        const imagePath=place.image
        try{
            const sess=await mongoose.startSession()
            sess.startTransaction()
            await place.remove({session:sess})
            user.places.pull(place)
            await user.save({session:sess})
            await sess.commitTransaction()
        }
        catch(err)
        {
            // res.status(500).json({message:err.message})
            return next(new HttpError(err.message,500))
        }
        fs.unlink(imagePath, err => {
            console.log(err)
          });
        console.log('Place deleted')
        res.status(200).json({message:'Place Deleted'})
    }
    catch(err)
    {
        // res.status(500).json({message:err.message})
        return next(new HttpError(err.message,500))
    }
}

exports.getPlaceByPid=getPlaceByPid
exports.getPlacesByUid=getPlacesByUid
exports.createPlace=createPlace
exports.updatePlace=updatePlace
exports.deletePlace=deletePlace