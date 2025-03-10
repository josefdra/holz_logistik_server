const mongoose = require('mongoose')

const locationSchema = new mongoose.Schema({
    _id: { type: Number, required: true },
    version: { type: Number, required: true, defaul: 0 },
    userId: { type: Number, required: true },
    lastEdited: { type: Date, required: true },
    latitude: { type: Number, required: true },
    longitude: { type: Number, required: true },
    partieNr: { type: String, required: true },
    contract: { type: String },
    additionalInfo: { type: String },
    access: { type: String },
    sawmill: { type: String },
    oversizeSawmill: { type: String },
    normalQuantity: { type: Number },
    oversizeQuantity: { type: Number },
    pieceCount: { type: Number, required: true },
    photos: [{ 
        fileId: mongoose.Schema.Types.ObjectId,
        filename: String 
    }]
}, {
    _id: false,
    timestamps: false
})

module.exports = mongoose.model('Location', locationSchema)