// App.js
import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Link } from 'react-router-dom';
import LocationsPage from './pages/LocationsPage';
import ShipmentsPage from './pages/ShipmentsPage';
import UsersPage from './pages/UsersPage';
import './App.css';

function App() {
  return (
    <Router>
      <div className="app">
        <header>
          <h1>Forest Database Management</h1>
          <nav>
            <Link to="/" className="nav-link">Locations</Link>
            <Link to="/shipments" className="nav-link">Shipments</Link>
            <Link to="/users" className="nav-link">Users</Link>
          </nav>
        </header>
        <main>
          <Routes>
            <Route path="/" element={<LocationsPage />} />
            <Route path="/shipments" element={<ShipmentsPage />} />
            <Route path="/users" element={<UsersPage />} />
          </Routes>
        </main>
        <footer>
          <p>&copy; 2025 Forest Database Management</p>
        </footer>
      </div>
    </Router>
  );
}

export default App;