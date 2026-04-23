import { API } from './api.js';

document.addEventListener('DOMContentLoaded', () => {
    const loginScreen = document.getElementById('login-screen');
    const journalScreen = document.getElementById('journal-screen');
    const loginBtn = document.getElementById('login-btn');
    const passwordInput = document.getElementById('password-input');
    const loginError = document.getElementById('login-error');
    const entriesList = document.getElementById('entries-list');

    // התחברות
    loginBtn.addEventListener('click', async () => {
        const isValid = await API.login(passwordInput.value);
        if (isValid) {
            loginScreen.style.display = 'none';
            journalScreen.style.display = 'block';
            loadEntries();
        } else {
            loginError.style.display = 'block';
        }
    });

    // טעינת רשומות
    async function loadEntries() {
        const entries = await API.getEntries();
        entriesList.innerHTML = entries.map(e => `
            <div class="entry" style="border: 1px solid #ccc; margin: 10px 0; padding: 10px;">
                <p>${e.content}</p>
            </div>
        `).join('');
    }
});