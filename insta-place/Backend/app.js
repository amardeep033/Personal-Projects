const express=require('express')
const app=express()
const fs=require('fs')
const path=require('path')

const mongoose=require('mongoose')
mongoose.connect('mongodb+srv://amardeep033:<password>@cluster0.4yoz0.mongodb.net/mern?retryWrites=true&w=majority',{useNewUrlParser:true})
const db=mongoose.connection
db.on('error',error=>console.log(error))
db.once('open',()=>console.log('Connected'))


app.use(express.json())

app.use('/uploads/images',express.static(path.join('uploads','images')))

app.use((req,res,next)=>{
    res.setHeader('Access-Control-Allow-Origin','*')
    res.setHeader('Access-Control-Allow-Headers','Content-Type,Origin,X-Requested-With,Accept,authorization')
    res.setHeader('Access-Control-Allow-Methods','GET,POST,PATCH,DELETE')
    next()
})

const HttpError=require('./models/http-error')

const placesRoutes=require('./routes/places-routes')
app.use('/api/places',placesRoutes)

const usersRoutes=require('./routes/users-routes')
app.use('/api/users',usersRoutes)

app.use((req,res,next)=>{
    const error=new HttpError('route not found',404)
    return next(error)
})

app.use((error,req,res,next)=>{
    if(req.file)
        fs.unlink(req.file.path,err=>{console.log(err)})
    if(res.headerSent)
        return next(error) //????
    res.status(error.code||500)
    res.json({message:error.message||'Unknown server error'})
})

app.listen(5000,()=>console.log('SERVER STARTED'))
