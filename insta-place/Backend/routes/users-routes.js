const express=require('express')
const router=express.Router()
const fileUpload=require('../middleware/file-upload')

const { check } = require('express-validator')

const usercontroller=require('../controllers/users-controller')

router.get('/',usercontroller.getAllUsers)

router.post('/signup',fileUpload.single('image'),[check('name').not().isEmpty(),check('email').normalizeEmail().isEmail() ,check('password').isLength({ min: 4 })],usercontroller.userSignup)
//image key in req

router.post('/login',usercontroller.userLogin)

module.exports=router