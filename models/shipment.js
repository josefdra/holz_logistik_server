const mongoose = require('mongoose')

const shipmentSchema = new mongoose.Schema({
    _id: { type: Number, required: true },
    version: { type: Number, required: true, defaul: 0 },
    userId: { type: Number, required: true },
    locationId: { type: Number, required: true },
    date: { type: Date, required: true },
    contract: { type: String },
    additionalInfo: { type: String },
    sawmill: { type: String, required: true },
    normalQuantity: { type: Number },
    oversizeQuantity: { type: Number },
    pieceCount: { type: Number, required: true }
}, {
    _id: false,
    timestamps: false
})

module.exports = mongoose.model('Shipment', shipmentSchema)