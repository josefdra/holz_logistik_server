import React, { useState, useEffect } from 'react';
import UserForm from '../components/UserForm';
import Modal from '../components/Modal';
import { fetchUsers, deleteUser } from '../services/api';

function UsersPage() {
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [showModal, setShowModal] = useState(false);
  const [currentUser, setCurrentUser] = useState(null);

  useEffect(() => {
    loadUsers();
  }, []);

  const loadUsers = async () => {
    setLoading(true);
    try {
      const data = await fetchUsers();
      setUsers(data);
      setError(null);
    } catch (err) {
      setError('Failed to load users. Please try again later.');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleEdit = (user) => {
    setCurrentUser(user);
    setShowModal(true);
  };

  const handleDelete = async (id) => {
    if (window.confirm('Are you sure you want to delete this user?')) {
      try {
        await deleteUser(id);
        setUsers(users.filter(user => user._id !== id));
      } catch (err) {
        setError('Failed to delete user. Please try again.');
        console.error(err);
      }
    }
  };

  const handleFormSubmit = () => {
    setShowModal(false);
    setCurrentUser(null);
    loadUsers();
  };

  return (
    <div>
      <div className="page-header">
        <h2 className="page-title">Users</h2>
        <button className="button" onClick={() => setShowModal(true)}>Add New User</button>
      </div>

      {error && <p className="error">{error}</p>}

      {loading ? (
        <p>Loading users...</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Name</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {users.length === 0 ? (
              <tr>
                <td colSpan="3" style={{ textAlign: 'center' }}>No users found</td>
              </tr>
            ) : (
              users.map(user => (
                <tr key={user._id}>
                  <td>{user._id}</td>
                  <td>{user.name}</td>
                  <td className="actions">
                    <button className="button" onClick={() => handleEdit(user)}>Edit</button>
                    <button className="button button-danger" onClick={() => handleDelete(user._id)}>Delete</button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      )}

      <Modal show={showModal} onClose={() => {
        setShowModal(false);
        setCurrentUser(null);
      }} title={currentUser ? 'Edit User' : 'Add New User'}>
        <UserForm 
          user={currentUser} 
          onSubmitSuccess={handleFormSubmit} 
        />
      </Modal>
    </div>
  );
}

export default UsersPage;