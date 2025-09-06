import './styles/main.scss';
import { App } from './app';

// Wait for DOM to be ready
document.addEventListener('DOMContentLoaded', () => {
    const app = new App();
    app.init();
});

// Handle window events
window.addEventListener('beforeunload', () => {
    // Cleanup
});