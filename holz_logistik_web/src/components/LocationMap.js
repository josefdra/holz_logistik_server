import React, { useState } from 'react';

function LocationMap({ locations, onEdit, onDelete }) {
  const [selectedLocation, setSelectedLocation] = useState(null);
  
  // This is a placeholder for a real map component
  // In a real application, you would use a library like Leaflet or Google Maps
  // For now, we'll create a simple visual representation
  
  const mapWidth = 800;
  const mapHeight = 500;
  
  // Find min/max lat/long to normalize coordinates
  const bounds = locations.reduce((acc, loc) => {
    return {
      minLat: Math.min(acc.minLat, loc.latitude),
      maxLat: Math.max(acc.maxLat, loc.latitude),
      minLng: Math.min(acc.minLng, loc.longitude),
      maxLng: Math.max(acc.maxLng, loc.longitude)
    };
  }, { 
    minLat: locations.length ? locations[0].latitude : 0, 
    maxLat: locations.length ? locations[0].latitude : 1,
    minLng: locations.length ? locations[0].longitude : 0,
    maxLng: locations.length ? locations[0].longitude : 1
  });
  
  // Add some padding
  const latRange = (bounds.maxLat - bounds.minLat) || 1;
  const lngRange = (bounds.maxLng - bounds.minLng) || 1;
  
  bounds.minLat -= latRange * 0.05;
  bounds.maxLat += latRange * 0.05;
  bounds.minLng -= lngRange * 0.05;
  bounds.maxLng += lngRange * 0.05;
  
  // Convert lat/lng to x/y coordinates
  const getXY = (lat, lng) => {
    const x = ((lng - bounds.minLng) / (bounds.maxLng - bounds.minLng)) * mapWidth;
    const y = ((bounds.maxLat - lat) / (bounds.maxLat - bounds.minLat)) * mapHeight;
    return { x, y };
  };

  return (
    <div className="map-container">
      {locations.length === 0 ? (
        <p style={{ textAlign: 'center' }}>No locations to display on the map</p>
      ) : (
        <>
          <div className="map" style={{ width: mapWidth, height: mapHeight, border: '1px solid #ccc', position: 'relative', backgroundColor: '#f3f3f3' }}>
            {/* Simple coordinate grid */}
            {[...Array(5)].map((_, i) => (
              <div key={`grid-h-${i}`} style={{ 
                position: 'absolute', 
                left: 0, 
                right: 0, 
                top: i * (mapHeight / 4), 
                height: 1, 
                backgroundColor: '#ddd' 
              }} />
            ))}
            {[...Array(5)].map((_, i) => (
              <div key={`grid-v-${i}`} style={{ 
                position: 'absolute', 
                top: 0, 
                bottom: 0, 
                left: i * (mapWidth / 4), 
                width: 1, 
                backgroundColor: '#ddd' 
              }} />
            ))}
            
            {/* Location markers */}
            {locations.map(location => {
              const { x, y } = getXY(location.latitude, location.longitude);
              return (
                <div 
                  key={location._id}
                  className="location-marker"
                  style={{
                    position: 'absolute',
                    left: x,
                    top: y,
                    width: 10,
                    height: 10,
                    borderRadius: '50%',
                    backgroundColor: '#3498db',
                    transform: 'translate(-50%, -50%)',
                    cursor: 'pointer',
                    zIndex: selectedLocation === location._id ? 10 : 1
                  }}
                  onClick={() => setSelectedLocation(location._id === selectedLocation ? null : location._id)}
                  title={`${location.partieNr} - ${location.totalQuantity} units`}
                />
              );
            })}
            
            {/* Selected location info */}
            {selectedLocation && (() => {
              const location = locations.find(l => l._id === selectedLocation);
              if (!location) return null;
              
              const { x, y } = getXY(location.latitude, location.longitude);
              const boxWidth = 200;
              const boxHeight = 180;
              
              // Adjust position to keep box in viewport
              let boxX = x + 15;
              let boxY = y - 10;
              
              if (boxX + boxWidth > mapWidth) boxX = x - boxWidth - 15;
              if (boxY + boxHeight > mapHeight) boxY = y - boxHeight;
              if (boxY < 0) boxY = 0;
              
              return (
                <div
                  className="location-info"
                  style={{
                    position: 'absolute',
                    left: boxX,
                    top: boxY,
                    width: boxWidth,
                    backgroundColor: 'white',
                    borderRadius: 4,
                    boxShadow: '0 2px 10px rgba(0,0,0,0.2)',
                    padding: 10,
                    zIndex: 100
                  }}
                >
                  <h4 style={{ margin: '0 0 5px 0' }}>Location {location.partieNr}</h4>
                  <p style={{ margin: '5px 0', fontSize: 14 }}>
                    <strong>Coordinates:</strong> {location.latitude.toFixed(4)}, {location.longitude.toFixed(4)}
                  </p>
                  <p style={{ margin: '5px 0', fontSize: 14 }}>
                    <strong>Quantity:</strong> {location.totalQuantity} units
                  </p>
                  <p style={{ margin: '5px 0', fontSize: 14 }}>
                    <strong>Pieces:</strong> {location.pieceCount}
                  </p>
                  <p style={{ margin: '5px 0', fontSize: 14 }}>
                    <strong>Sawmill:</strong> {location.sawmill || 'N/A'}
                  </p>
                  <div style={{ marginTop: 10, display: 'flex', gap: 5 }}>
                    <button 
                      className="button" 
                      style={{ padding: '3px 8px', fontSize: 12 }}
                      onClick={() => onEdit(location)}
                    >
                      Edit
                    </button>
                    <button 
                      className="button button-danger" 
                      style={{ padding: '3px 8px', fontSize: 12 }}
                      onClick={() => onDelete(location._id)}
                    >
                      Delete
                    </button>
                  </div>
                </div>
              );
            })()}
          </div>
          
          <div className="map-legend" style={{ marginTop: 10, fontSize: 14 }}>
            <p><strong>Coordinates:</strong> Map shows {bounds.minLat.toFixed(4)}° to {bounds.maxLat.toFixed(4)}° latitude, {bounds.minLng.toFixed(4)}° to {bounds.maxLng.toFixed(4)}° longitude</p>
            <p><strong>Click on a marker</strong> to see location details</p>
          </div>
        </>
      )}
    </div>
  );
}

export default LocationMap;