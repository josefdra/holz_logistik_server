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
        userId: req.body.userId,
        lastEdited: req.body.lastEdited,
        contract: req.body.contract,
        additionalInfo: req.body.additionalInfo,
        sawmill: req.body.sawmill,
        totalQuantity: req.body.totalQuantity,
        oversizedQuantity: req.body.oversizedQuantity,
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
    if (req.body.userId != null) res.shipment.userId = req.body.userId
    if (req.body.lastEdited != null) res.shipment.lastEdited = req.body.lastEdited
    if (req.body.contract != null) res.shipment.contract = req.body.contract
    if (req.body.additionalInfo != null) res.shipment.additionalInfo = req.body.additionalInfo
    if (req.body.sawmill != null) res.shipment.sawmill = req.body.sawmill
    if (req.body.totalQuantity != null) res.shipment.totalQuantity = req.body.totalQuantity
    if (req.body.oversizedQuantity != null) res.shipment.oversizedQuantity = req.body.oversizedQuantity
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