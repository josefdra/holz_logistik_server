const mongoose = require('mongoose')

const shipmentSchema = new mongoose.Schema({
    _id: { type: Number, required: true },
    userId: { type: Number, required: true },
    lastEdited: { type: Date, required: true },
    contract: { type: String },
    additionalInfo: { type: String },
    sawmill: { type: String, required: true },
    totalQuantity: { type: Number, required: true },
    oversizedQuantity: { type: Number },
    pieceCount: { type: Number, required: true }
}, {
    _id: false,
    timestamps: false
})

module.exports = mongoose.model('Shipment', shipmentSchema)