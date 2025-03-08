require('dotenv').config()

const express = require('express')
const cors = require('cors')
const app = express()
const mongoose = require('mongoose')

mongoose.connect(process.env.DATABASE_URL)
const db = mongoose.connection
db.on('error', (error) => console.error(error))
db.once('open', () => console.log('Connected to database'))

app.use(cors())
app.use(express.json())

const usersRouter = require('./routes/users')
app.use('/users', usersRouter)
const locationsRouter = require('./routes/locations')
app.use('/locations', locationsRouter)
const shipmentsRouter = require('./routes/shipments')
app.use('/shipments', shipmentsRouter)

app.listen(3000, () => console.log('Server Started'))
