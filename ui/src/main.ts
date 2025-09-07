import '@/styles/main.scss';
import { App } from './app';

const mount = async () => {
  const root = document.getElementById('app')!;
  const app = new App(root);
  await app.mount();
};

mount();
