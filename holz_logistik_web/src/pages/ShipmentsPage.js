import React, { useState, useEffect } from 'react';
import ShipmentForm from '../components/ShipmentForm';
import Modal from '../components/Modal';
import { fetchShipments, deleteShipment } from '../services/api';

function ShipmentsPage() {
  const [shipments, setShipments] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [showModal, setShowModal] = useState(false);
  const [currentShipment, setCurrentShipment] = useState(null);

  useEffect(() => {
    loadShipments();
  }, []);

  const loadShipments = async () => {
    setLoading(true);
    try {
      const data = await fetchShipments();
      setShipments(data);
      setError(null);
    } catch (err) {
      setError('Failed to load shipments. Please try again later.');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleEdit = (shipment) => {
    setCurrentShipment(shipment);
    setShowModal(true);
  };

  const handleDelete = async (id) => {
    if (window.confirm('Are you sure you want to delete this shipment?')) {
      try {
        await deleteShipment(id);
        setShipments(shipments.filter(shipment => shipment._id !== id));
      } catch (err) {
        setError('Failed to delete shipment. Please try again.');
        console.error(err);
      }
    }
  };

  const handleFormSubmit = () => {
    setShowModal(false);
    setCurrentShipment(null);
    loadShipments();
  };

  return (
    <div>
      <div className="page-header">
        <h2 className="page-title">Shipments</h2>
        <button className="button" onClick={() => setShowModal(true)}>Add New Shipment</button>
      </div>

      {error && <p className="error">{error}</p>}

      {loading ? (
        <p>Loading shipments...</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Contract</th>
              <th>Sawmill</th>
              <th>Total Quantity</th>
              <th>Oversized Quantity</th>
              <th>Piece Count</th>
              <th>Last Edited</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {shipments.length === 0 ? (
              <tr>
                <td colSpan="8" style={{ textAlign: 'center' }}>No shipments found</td>
              </tr>
            ) : (
              shipments.map(shipment => (
                <tr key={shipment._id}>
                  <td>{shipment._id}</td>
                  <td>{shipment.contract || 'N/A'}</td>
                  <td>{shipment.sawmill}</td>
                  <td>{shipment.totalQuantity}</td>
                  <td>{shipment.oversizedQuantity || 'N/A'}</td>
                  <td>{shipment.pieceCount}</td>
                  <td>{new Date(shipment.lastEdited).toLocaleDateString()}</td>
                  <td className="actions">
                    <button className="button" onClick={() => handleEdit(shipment)}>Edit</button>
                    <button className="button button-danger" onClick={() => handleDelete(shipment._id)}>Delete</button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      )}

      <Modal show={showModal} onClose={() => {
        setShowModal(false);
        setCurrentShipment(null);
      }} title={currentShipment ? 'Edit Shipment' : 'Add New Shipment'}>
        <ShipmentForm 
          shipment={currentShipment} 
          onSubmitSuccess={handleFormSubmit} 
        />
      </Modal>
    </div>
  );
}

export default ShipmentsPage;