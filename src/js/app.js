import { API } from './api.js';

document.addEventListener('DOMContentLoaded', async () => {
    // אלמנטים
    const setupScreen = document.getElementById('setup-screen');
    const loginScreen = document.getElementById('login-screen');
    const journalScreen = document.getElementById('journal-screen');
    const entriesList = document.getElementById('entries-list');

    // נבדוק האם הוגדרה סיסמה בעבר
    const isSetup = await API.checkIsSetup();
    if (isSetup) {
        loginScreen.style.display = 'flex';
    } else {
        setupScreen.style.display = 'flex';
    }

    // --- הרשמה ---
    document.getElementById('setup-btn').addEventListener('click', async () => {
        const pass = document.getElementById('setup-password').value;
        if (!pass) return;
        await API.setupPassword(pass);
        setupScreen.style.display = 'none';
        showJournal();
    });

    // --- התחברות ---
    document.getElementById('login-btn').addEventListener('click', async () => {
        const pass = document.getElementById('login-password').value;
        const success = await API.login(pass);
        if (success) {
            loginScreen.style.display = 'none';
            showJournal();
        } else {
            document.getElementById('login-error').style.display = 'block';
        }
    });

    // --- יומן ---
    async function showJournal() {
        journalScreen.style.display = 'flex';
        loadEntries();
    }

    document.getElementById('save-btn').addEventListener('click', async () => {
        const textElem = document.getElementById('entry-text');
        const content = textElem.value.trim();
        if (!content) return;

        const entryData = {
            content: content,
            type: document.getElementById('entry-type').value,
            category: document.getElementById('entry-category').value,
            rating: document.getElementById('entry-rating').value
        };

        await API.saveEntry(entryData);
        
        // איפוס טופס ורענון רשימה
        textElem.value = '';
        document.getElementById('entry-category').value = '';
        document.getElementById('entry-rating').value = '';
        loadEntries();
    });

    async function loadEntries() {
        const entries = await API.getEntries();
        entriesList.innerHTML = '';

        entries.forEach(item => {
            // פענוח ה-JSON הפנימי שחזר מה-Rust
            const data = JSON.parse(item.payload);
            const dateStr = new Date(item.timestamp).toLocaleString('he-IL');

            const div = document.createElement('div');
            div.className = `entry-card type-${data.type}`;
            div.innerHTML = `
                <div class="entry-meta">
                    <span>${dateStr}</span>
                    ${data.category ? `<span>• קטגוריה: ${data.category}</span>` : ''}
                    ${data.rating ? `<span>• דירוג: ${data.rating}</span>` : ''}
                </div>
                <div class="entry-content">${data.content}</div>
            `;
            entriesList.appendChild(div);
        });
    }
});
