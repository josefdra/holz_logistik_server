const express = require('express')
const router = express.Router()
const Shipment = require('../models/shipment')

// Getting all shipments
router.get('/', async (req, res) => {
    try {
        const shipment = await Shipment.find()
        res.json(shipment)
    } catch (err) {
        res.status(500).json({ message: err.message })
    }
})

// Getting one
router.get('/:id', getShipment, (req, res) => {
    res.json(res.shipment)
})

// Creating one
router.post('/', async (req, res) => {
    const shipment = new Shipment({
        _id: req.body.id,
        version: req.body.version,
        userId: req.body.userId,
        locationId: req.body.locationId,
        date: req.body.date,
        contract: req.body.contract,
        additionalInfo: req.body.additionalInfo,
        sawmill: req.body.sawmill,
        normalQuantity: req.body.normalQuantity,
        oversizeQuantity: req.body.oversizeQuantity,
        pieceCount: req.body.pieceCount
    })
    try {
        const newShipment = await shipment.save()
        res.status(201).json(newShipment)
    } catch (err) {
        res.status(400).json({ message: err.message })
    }
})

// Updating one
router.patch('/:id', getShipment, async (req, res) => {
    if (req.body.version != null) res.shipment.version = req.body.version
    if (req.body.userId != null) res.shipment.userId = req.body.userId
    if (req.body.locationId != null) res.shipment.locationId = req.body.locationId
    if (req.body.date != null) res.shipment.date = req.body.date
    if (req.body.contract != null) res.shipment.contract = req.body.contract
    if (req.body.additionalInfo != null) res.shipment.additionalInfo = req.body.additionalInfo
    if (req.body.sawmill != null) res.shipment.sawmill = req.body.sawmill
    if (req.body.normalQuantity != null) res.shipment.normalQuantity = req.body.normalQuantity
    if (req.body.oversizeQuantity != null) res.shipment.oversizeQuantity = req.body.oversizeQuantity
    if (req.body.pieceCount != null) res.shipment.pieceCount = req.body.pieceCount

    try {
        const updatedShipment = await res.shipment.save()
        res.json(updatedShipment)
    } catch (err) {
        res.status(400).json({ message: err.message })
    }
})

// Deleting one
router.delete('/:id', getShipment, async (req, res) => {
    try {
        await res.shipment.deleteOne()
        res.json({ message: 'Deleted Shipment' })
    } catch (err) {
        res.status(500).json({ message: err.message })
    }
})

async function getShipment(req, res, next) {
    let shipment
    try {
        shipment = await Shipment.findById(req.params.id)
        if (shipment == null) {
            return res.status(404).json({ message: 'Cannot find shipment' })
        }
    } catch (err) {
        return res.status(500).json({ message: err.message })
    }

    res.shipment = shipment
    next()
}

module.exports = router