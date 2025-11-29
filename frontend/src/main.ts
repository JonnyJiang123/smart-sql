import './app.css';
import App from './AppWithTabs.svelte';

const target = document.getElementById('app');

if (!target) {
  throw new Error('根元素 #app 未找到');
}

const app = new App({
  target
});

export default app;
