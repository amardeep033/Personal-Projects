const HttpError = require('../models/http-error')
const bcrypt = require('bcryptjs')
const jwt=require('jsonwebtoken')

const { validationResult } = require('express-validator')

const User = require('../models/USER')

const getAllUsers = async (req, res, next) => {
    try {
        console.log('Get users places')
        const users = await User.find({}, '-password')
        if (!users || users.length === 0)
            return next(new HttpError('No user available', 404))
        res.status(200).json({ users: users.map(u => u.toObject({ getters: true })) })
    }
    catch (err) {
        return next(new HttpError(err.message, 500))
    }
}

const userLogin = async (req, res, next) => {
    const { email, password } = req.body
    let existUser
    try {
        existUser = await User.findOne({ email: email })
    }
    catch (err) {
        return next(new HttpError(err.message, 500))
    }
    if (!existUser) {
        console.log('Invalid email')
        return next(new HttpError('Email doesnt exist', 401))
    }
    let validity
    try {
        validity = await bcrypt.compare(password,existUser.password)
    } 
    catch (err) 
    {
        return next(new HttpError(err.message, 500))
    }
    if (!validity) 
    {
        console.log('Invalid password')
        return next(new HttpError('Invalid password', 401))
    }
    let token
    try {
        token=jwt.sign({userId:existUser.id,email:existUser.email},'thisissecretkey',{expiresIn:'1h'})
    } catch (err) {
        return next(new HttpError(err.message, 500))
    }
    console.log('logged in')
    res.status(201).json({userId:existUser.id,email:existUser.email,token:token})
}

const userSignup = async (req, res, next) => {
    const valRes = validationResult(req)
    if (!valRes.isEmpty()) {
        console.log(valRes)
        return next(new HttpError('empty or invalid inputs', 422))
    }
    const { name, email, password } = req.body
    let existingUser
    try {
        existingUser = await User.findOne({ email: email })
    }
    catch (err) {
        return next(new HttpError(err.message, 500))
    }
    if (existingUser) {
        console.log("user already exist")
        return next(new HttpError('Email already exist', 422))
    }
    let hashedPassword
    try {
        hashedPassword = await bcrypt.hash(password, 12)
    }
    catch (err) {
        return next(new HttpError(err.message, 500))
    }
    const createdUser = new User({
        name, email, password: hashedPassword, places: [], image: req.file.path
    })
    try {
        await createdUser.save()
        console.log('User created')
    }
    catch (err) {
        // res.status(500).json({message:err.message})
        return next(new HttpError(err.message, 500))
    }
    let token
    try {
        token=jwt.sign({userId:createdUser.id,email:createdUser.email},'thisissecretkey',{expiresIn:'1h'})
    } catch (err) {
        return next(new HttpError(err.message, 500))
    }
    res.status(201).json({userId:createdUser.id,email:createdUser.email,token:token})
}

exports.getAllUsers = getAllUsers
exports.userSignup = userSignup
exports.userLogin = userLogin