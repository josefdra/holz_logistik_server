const API_URL = 'http://localhost:3000';

// Location APIs
export const fetchLocations = async () => {
  const response = await fetch(`${API_URL}/locations`);
  if (!response.ok) {
    throw new Error(`Failed to fetch locations: ${response.statusText}`);
  }
  return response.json();
};

export const fetchLocation = async (id) => {
  const response = await fetch(`${API_URL}/locations/${id}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch location: ${response.statusText}`);
  }
  return response.json();
};

export const createLocation = async (locationData) => {
  const response = await fetch(`${API_URL}/locations`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(locationData),
  });
  
  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || 'Failed to create location');
  }
  
  return response.json();
};

export const updateLocation = async (id, locationData) => {
  const response = await fetch(`${API_URL}/locations/${id}`, {
    method: 'PATCH',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(locationData),
  });
  
  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || 'Failed to update location');
  }
  
  return response.json();
};

export const deleteLocation = async (id) => {
  const response = await fetch(`${API_URL}/locations/${id}`, {
    method: 'DELETE',
  });
  
  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || 'Failed to delete location');
  }
  
  return true;
};

// Shipment APIs
export const fetchShipments = async () => {
  const response = await fetch(`${API_URL}/shipments`);
  if (!response.ok) {
    throw new Error(`Failed to fetch shipments: ${response.statusText}`);
  }
  return response.json();
};

export const fetchShipment = async (id) => {
  const response = await fetch(`${API_URL}/shipments/${id}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch shipment: ${response.statusText}`);
  }
  return response.json();
};

export const createShipment = async (shipmentData) => {
  const response = await fetch(`${API_URL}/shipments`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(shipmentData),
  });
  
  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || 'Failed to create shipment');
  }
  
  return response.json();
};

export const updateShipment = async (id, shipmentData) => {
  const response = await fetch(`${API_URL}/shipments/${id}`, {
    method: 'PATCH',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(shipmentData),
  });
  
  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || 'Failed to update shipment');
  }
  
  return response.json();
};

export const deleteShipment = async (id) => {
  const response = await fetch(`${API_URL}/shipments/${id}`, {
    method: 'DELETE',
  });
  
  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || 'Failed to delete shipment');
  }
  
  return true;
};

// User APIs
export const fetchUsers = async () => {
  const response = await fetch(`${API_URL}/users`);
  if (!response.ok) {
    throw new Error(`Failed to fetch users: ${response.statusText}`);
  }
  return response.json();
};

export const fetchUser = async (id) => {
  const response = await fetch(`${API_URL}/users/${id}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch user: ${response.statusText}`);
  }
  return response.json();
};

export const createUser = async (userData) => {
  const response = await fetch(`${API_URL}/users`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(userData),
  });
  
  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || 'Failed to create user');
  }
  
  return response.json();
};

export const updateUser = async (id, userData) => {
  const response = await fetch(`${API_URL}/users/${id}`, {
    method: 'PATCH',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(userData),
  });
  
  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || 'Failed to update user');
  }
  
  return response.json();
};

export const deleteUser = async (id) => {
  const response = await fetch(`${API_URL}/users/${id}`, {
    method: 'DELETE',
  });
  
  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || 'Failed to delete user');
  }
  
  return true;
};
