import axios from 'axios';

const api = axios.create({
    baseURL: 'http://localhost:8080', // Matches your Axum backend port
    headers: {
        'Content-Type': 'application/json',
    },
});

// Request Interceptor: Auto-attach JWT
api.interceptors.request.use((config) => {
    if (typeof window !== 'undefined') {
        const token = localStorage.getItem('token');
        if (token) {
            config.headers.Authorization = `Bearer ${token}`;
        }
    }
    return config;
});

export default api;