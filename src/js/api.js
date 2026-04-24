const isTauri = window.__TAURI__ !== undefined;
const invoke = isTauri ? window.__TAURI__.invoke : null;

// Mock נתונים לסביבת פיתוח בדפדפן
let mockEntries = JSON.parse(localStorage.getItem('mock_entries') || '[]');
let mockIsSetup = localStorage.getItem('mock_is_setup') === 'true';

export const API = {
    checkIsSetup: async () => {
        if (isTauri) return await invoke('check_is_setup');
        return mockIsSetup;
    },
    setupPassword: async (password) => {
        if (isTauri) return await invoke('setup_password', { password });
        localStorage.setItem('mock_is_setup', 'true');
        return true;
    },
    login: async (password) => {
        if (isTauri) return await invoke('login', { password });
        return password === "1234"; // בדפדפן, 1234 עובד תמיד
    },
    saveEntry: async (entryObj) => {
        const timestamp = Date.now();
        const payloadJson = JSON.stringify(entryObj); // הופכים את האובייקט לטקסט בשביל ה-Rust
        
        if (isTauri) {
            return await invoke('save_entry', { timestamp, payloadJson });
        } else {
            const newEntry = { id: Date.now(), timestamp, payload: payloadJson };
            mockEntries.unshift(newEntry); // הוספה להתחלה
            localStorage.setItem('mock_entries', JSON.stringify(mockEntries));
            return newEntry.id;
        }
    },
    getEntries: async () => {
        if (isTauri) return await invoke('get_entries');
        return mockEntries;
    }
};
