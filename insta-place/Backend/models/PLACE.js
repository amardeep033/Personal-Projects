const mongoose = require('mongoose');
// const User=require('../models/USER')
const Schema = mongoose.Schema;


const placeSchema = new Schema({
    title: { type: String, required: true },
    description: { type: String, required: true },
    image: { type: String, required: true },
    address: { type: String, required: true },
    creatorId: { type: mongoose.Types.ObjectId, required: true,ref:'User'}
});

module.exports = mongoose.model('Place', placeSchema);