// בדיקה האם אנחנו רצים בתוך Tauri או בדפדפן רגיל
const isTauri = window.__TAURI__ !== undefined;
const invoke = isTauri ? window.__TAURI__.invoke : null;

export const API = {
    login: async (password) => {
        if (isTauri) {
            return await invoke('check_auth', { password });
        } else {
            // Mock לפיתוח מקומי
            console.log("Mock: Checking password...");
            return password === "1234"; // סיסמת דמה לפיתוח
        }
    },

    getEntries: async () => {
        if (isTauri) {
            return await invoke('get_entries');
        } else {
            // Mock לפיתוח מקומי
            return [
                { id: 1, content: "פיילוט עובד! זה נתון מה-Mock", type: "positive", date: Date.now() }
            ];
        }
    }
};