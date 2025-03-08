const mongoose = require('mongoose')

const locationSchema = new mongoose.Schema({
    _id: { type: Number, required: true },
    userId: { type: Number, required: true },
    lastEdited: { type: Date, required: true },
    latitude: { type: Number, required: true },
    longitude: { type: Number, required: true },
    partieNr: { type: String, required: true },
    contract: { type: String },
    additionalInfo: { type: String },
    access: { type: String },
    sawmill: { type: String },
    overSizeSawmill: { type: String },
    totalQuantity: { type: Number, required: true },
    oversizedQuantity: { type: Number },
    pieceCount: { type: Number, required: true }
}, {
    _id: false,
    timestamps: false
})

module.exports = mongoose.model('Location', locationSchema)