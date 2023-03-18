import App from './App.svelte';

const app = new App({
    // Safe since we know it exists
    target: document.getElementById('app') as Element,
});

export default app;
