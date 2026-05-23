const express = require('express')
const router = express.Router()
const fileUpload=require('../middleware/file-upload')
const authToken=require('../middleware/auth-token')

const { check } = require('express-validator')

const placecontroller = require('../controllers/places-controllers')

router.get('/:pid', placecontroller.getPlaceByPid)

router.get('/user/:uid', placecontroller.getPlacesByUid)

router.use(authToken)

router.patch('/:pid', [check('title').not().isEmpty(), check('description').isLength({ min: 5 })], placecontroller.updatePlace)

router.post('/', fileUpload.single('image'),[check('title').not().isEmpty(), check('description').isLength({ min: 5 }), check('address').not().isEmpty()], placecontroller.createPlace)

router.delete('/:pid', placecontroller.deletePlace)

module.exports = router