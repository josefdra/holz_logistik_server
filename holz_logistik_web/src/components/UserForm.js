import React, { useState, useEffect } from 'react';
import { createUser, updateUser } from '../services/api';

function UserForm({ user, onSubmitSuccess }) {
  const [formData, setFormData] = useState({
    name: ''
  });
  
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (user) {
      setFormData({
        name: user.name
      });
    }
  }, [user]);

  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData({
      ...formData,
      [name]: value
    });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    // Validation
    if (!formData.name) {
      setError('Please enter a name');
      return;
    }
    
    setLoading(true);
    setError(null);
    
    try {
      if (user) {
        await updateUser(user._id, formData);
      } else {
        await createUser(formData);
      }
      onSubmitSuccess();
    } catch (err) {
      setError(err.message || 'Failed to save user');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <div className="form-group">
        <label htmlFor="name">Name *</label>
        <input 
          type="text" 
          id="name" 
          name="name" 
          value={formData.name} 
          onChange={handleChange}
          required
        />
      </div>

      {error && <p className="error">{error}</p>}

      <div className="form-buttons">
        <button 
          type="submit" 
          className="button" 
          disabled={loading}
        >
          {loading ? 'Saving...' : (user ? 'Update User' : 'Create User')}
        </button>
      </div>
    </form>
  );
}

export default UserForm;