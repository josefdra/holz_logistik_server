import React, { useState, useEffect } from 'react';
import { createShipment, updateShipment, fetchUsers } from '../services/api';

function ShipmentForm({ shipment, onSubmitSuccess }) {
  const [formData, setFormData] = useState({
    id: '',
    userId: '',
    lastEdited: new Date().toISOString(),
    contract: '',
    additionalInfo: '',
    sawmill: '',
    totalQuantity: '',
    oversizedQuantity: '',
    pieceCount: ''
  });
  
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (shipment) {
      setFormData({
        id: shipment._id,
        userId: shipment.userId,
        lastEdited: new Date().toISOString(),
        contract: shipment.contract || '',
        additionalInfo: shipment.additionalInfo || '',
        sawmill: shipment.sawmill,
        totalQuantity: shipment.totalQuantity,
        oversizedQuantity: shipment.oversizedQuantity || '',
        pieceCount: shipment.pieceCount
      });
    } else {
      // Generate new ID for new shipments
      setFormData({
        ...formData,
        id: Math.floor(Math.random() * 1000000)
      });
    }

    // Load users for the dropdown
    loadUsers();
  }, [shipment]);

  const loadUsers = async () => {
    try {
      const data = await fetchUsers();
      setUsers(data);
    } catch (err) {
      console.error('Failed to load users', err);
    }
  };

  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData({
      ...formData,
      [name]: name === 'totalQuantity' || name === 'oversizedQuantity' || name === 'pieceCount' || name === 'userId' || name === 'id'
        ? Number(value) 
        : value
    });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    // Validation
    if (!formData.userId || !formData.sawmill || !formData.totalQuantity || !formData.pieceCount) {
      setError('Please fill in all required fields');
      return;
    }
    
    setLoading(true);
    setError(null);
    
    try {
      if (shipment) {
        await updateShipment(shipment._id, formData);
      } else {
        await createShipment(formData);
      }
      onSubmitSuccess();
    } catch (err) {
      setError(err.message || 'Failed to save shipment');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <div className="form-grid">
        <div className="form-group">
          <label htmlFor="userId">User *</label>
          <select 
            id="userId" 
            name="userId" 
            value={formData.userId} 
            onChange={handleChange}
            required
          >
            <option value="">Select User</option>
            {users.map(user => (
              <option key={user._id} value={user._id}>{user.name}</option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label htmlFor="contract">Contract</label>
          <input 
            type="text" 
            id="contract" 
            name="contract" 
            value={formData.contract} 
            onChange={handleChange}
          />
        </div>

        <div className="form-group">
          <label htmlFor="sawmill">Sawmill *</label>
          <input 
            type="text" 
            id="sawmill" 
            name="sawmill"
            value={formData.sawmill} 
            onChange={handleChange}
            required
          />
        </div>

        <div className="form-group">
          <label htmlFor="totalQuantity">Total Quantity *</label>
          <input 
            type="number" 
            id="totalQuantity" 
            name="totalQuantity" 
            value={formData.totalQuantity} 
            onChange={handleChange}
            required
          />
        </div>

        <div className="form-group">
          <label htmlFor="oversizedQuantity">Oversized Quantity</label>
          <input 
            type="number" 
            id="oversizedQuantity" 
            name="oversizedQuantity" 
            value={formData.oversizedQuantity} 
            onChange={handleChange}
          />
        </div>

        <div className="form-group">
          <label htmlFor="pieceCount">Piece Count *</label>
          <input 
            type="number" 
            id="pieceCount" 
            name="pieceCount" 
            value={formData.pieceCount} 
            onChange={handleChange}
            required
          />
        </div>
      </div>

      <div className="form-group">
        <label htmlFor="additionalInfo">Additional Information</label>
        <textarea 
          id="additionalInfo" 
          name="additionalInfo" 
          value={formData.additionalInfo} 
          onChange={handleChange}
          rows="3"
        />
      </div>

      {error && <p className="error">{error}</p>}

      <div className="form-buttons">
        <button 
          type="submit" 
          className="button" 
          disabled={loading}
        >
          {loading ? 'Saving...' : (shipment ? 'Update Shipment' : 'Create Shipment')}
        </button>
      </div>
    </form>
  );
}

export default ShipmentForm;