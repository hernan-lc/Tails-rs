// examples/public/app.js

// Fetch and display users when the page loads
async function fetchUsers() {
    try {
        const response = await fetch('/api/users');
        const users = await response.json();

        const listElement = document.getElementById('users-list');
        listElement.innerHTML = users.map(user =>
            `<li><strong>${user.name}</strong> - ${user.role}</li>`
        ).join('');
    } catch (error) {
        console.error("Failed to fetch users:", error);
    }
}

// Handle submitting the form to create a new user
async function createUser(event) {
    event.preventDefault();

    const nameInput = document.getElementById('user-name').value;
    const roleInput = document.getElementById('user-role').value;

    try {
        const response = await fetch('/api/users', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name: nameInput, role: roleInput })
        });

        if (response.ok) {
            // Clear inputs and refresh list
            document.getElementById('user-name').value = '';
            document.getElementById('user-role').value = '';
            fetchUsers();
        } else {
            const errData = await response.json();
            alert(`Error: ${errData.error}`);
        }
    } catch (error) {
        console.error("Failed to create user:", error);
    }
}

document.addEventListener('DOMContentLoaded', () => {
    fetchUsers();
    document.getElementById('user-form').addEventListener('submit', createUser);
});
