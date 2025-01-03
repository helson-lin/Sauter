import { Store } from '@tauri-apps/plugin-store';

const store = new Store('.settings.dat');

export default store;