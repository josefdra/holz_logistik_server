const express = require('express')
const router = express.Router()
const Location = require('../models/location')

// Getting all locations
router.get('/', async (req, res) => {
    try {
        const location = await Location.find()
        res.json(location)
    } catch (err) {
        res.status(500).json({ message: err.message })
    }
})

// Getting one
router.get('/:id', getLocation, (req, res) => {
    res.json(res.location)
})

// Creating one
router.post('/', async (req, res) => {
    const location = new Location({
        _id: req.body.id,
        userId: req.body.userId,
        lastEdited: req.body.lastEdited,
        latitude: req.body.latitude,
        longitude: req.body.longitude,
        partieNr: req.body.partieNr,
        contract: req.body.contract,
        additionalInfo: req.body.additionalInfo,
        access: req.body.access,
        sawmill: req.body.sawmill,
        overSizeSawmill: req.body.overSizeSawmill,
        totalQuantity: req.body.totalQuantity,
        oversizedQuantity: req.body.oversizedQuantity,
        pieceCount: req.body.pieceCount
    })
    try {
        const newLocation = await location.save()
        res.status(201).json(newLocation)
    } catch (err) {
        res.status(400).json({ message: err.message })
    }
})

// Updating one
router.patch('/:id', getLocation, async (req, res) => {
    if (req.body.userId != null) res.location.userId = req.body.userId
    if (req.body.lastEdited != null) res.location.lastEdited = req.body.lastEdited
    if (req.body.latitude != null) res.location.latitude = req.body.latitude
    if (req.body.longitude != null) res.location.longitude = req.body.longitude
    if (req.body.partieNr != null) res.location.partieNr = req.body.partieNr
    if (req.body.contract != null) res.location.contract = req.body.contract
    if (req.body.additionalInfo != null) res.location.additionalInfo = req.body.additionalInfo
    if (req.body.access != null) res.location.access = req.body.access
    if (req.body.sawmill != null) res.location.sawmill = req.body.sawmill
    if (req.body.overSizeSawmill != null) res.location.overSizeSawmill = req.body.overSizeSawmill
    if (req.body.totalQuantity != null) res.location.totalQuantity = req.body.totalQuantity
    if (req.body.oversizedQuantity != null) res.location.oversizedQuantity = req.body.oversizedQuantity
    if (req.body.pieceCount != null) res.location.pieceCount = req.body.pieceCount

    try {
        const updateLocation = await res.location.save()
        res.json(updateLocation)
    } catch (err) {
        res.status(400).json({ message: err.message })
    }
})

// Deleting one
router.delete('/:id', getLocation, async (req, res) => {
    try {
        await res.location.deleteOne()
        res.json({ message: 'Deleted Location' })
    } catch (err) {
        res.status(500).json({ message: err.message })
    }
})

async function getLocation(req, res, next) {
    let location
    try {
        location = await Location.findById(req.params.id)
        if (location == null) {
            return res.status(404).json({ message: 'Cannot find location' })
        }
    } catch (err) {
        return res.status(500).json({ message: err.message })
    }

    res.location = location
    next()
}

module.exports = router