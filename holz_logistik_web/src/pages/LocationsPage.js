import React, { useState, useEffect } from 'react';
import LocationForm from '../components/LocationForm';
import LocationMap from '../components/LocationMap';
import Modal from '../components/Modal';
import { fetchLocations, deleteLocation } from '../services/api';

function LocationsPage() {
  const [locations, setLocations] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [showModal, setShowModal] = useState(false);
  const [currentLocation, setCurrentLocation] = useState(null);
  const [viewMode, setViewMode] = useState('table'); // 'table' or 'map'

  useEffect(() => {
    loadLocations();
  }, []);

  const loadLocations = async () => {
    setLoading(true);
    try {
      const data = await fetchLocations();
      setLocations(data);
      setError(null);
    } catch (err) {
      setError('Failed to load locations. Please try again later.');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleEdit = (location) => {
    setCurrentLocation(location);
    setShowModal(true);
  };

  const handleDelete = async (id) => {
    if (window.confirm('Are you sure you want to delete this location?')) {
      try {
        await deleteLocation(id);
        setLocations(locations.filter(location => location._id !== id));
      } catch (err) {
        setError('Failed to delete location. Please try again.');
        console.error(err);
      }
    }
  };

  const handleFormSubmit = () => {
    setShowModal(false);
    setCurrentLocation(null);
    loadLocations();
  };

  return (
    <div>
      <div className="page-header">
        <h2 className="page-title">Locations</h2>
        <div className="header-actions">
          <div className="view-toggles">
            <button 
              className={`button ${viewMode === 'table' ? 'active' : ''}`}
              onClick={() => setViewMode('table')}
            >
              Table View
            </button>
            <button 
              className={`button ${viewMode === 'map' ? 'active' : ''}`}
              onClick={() => setViewMode('map')}
            >
              Map View
            </button>
          </div>
          <button className="button" onClick={() => setShowModal(true)}>Add New Location</button>
        </div>
      </div>

      {error && <p className="error">{error}</p>}

      {loading ? (
        <p>Loading locations...</p>
      ) : (
        <>
          {viewMode === 'map' ? (
            <LocationMap 
              locations={locations} 
              onEdit={handleEdit} 
              onDelete={handleDelete} 
            />
          ) : (
            <table className="data-table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Party Nr</th>
                  <th>Location</th>
                  <th>Quantity</th>
                  <th>Piece Count</th>
                  <th>Sawmill</th>
                  <th>Last Edited</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {locations.length === 0 ? (
                  <tr>
                    <td colSpan="8" style={{ textAlign: 'center' }}>No locations found</td>
                  </tr>
                ) : (
                  locations.map(location => (
                    <tr key={location._id}>
                      <td>{location._id}</td>
                      <td>{location.partieNr}</td>
                      <td>{`${location.latitude.toFixed(4)}, ${location.longitude.toFixed(4)}`}</td>
                      <td>{location.totalQuantity}</td>
                      <td>{location.pieceCount}</td>
                      <td>{location.sawmill || 'N/A'}</td>
                      <td>{new Date(location.lastEdited).toLocaleDateString()}</td>
                      <td className="actions">
                        <button className="button" onClick={() => handleEdit(location)}>Edit</button>
                        <button className="button button-danger" onClick={() => handleDelete(location._id)}>Delete</button>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          )}
        </>
      )}

      <Modal show={showModal} onClose={() => {
        setShowModal(false);
        setCurrentLocation(null);
      }} title={currentLocation ? 'Edit Location' : 'Add New Location'}>
        <LocationForm 
          location={currentLocation} 
          onSubmitSuccess={handleFormSubmit} 
        />
      </Modal>
    </div>
  );
}

export default LocationsPage;