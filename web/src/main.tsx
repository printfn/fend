import { StrictMode } from 'react';
import ReactDOM from 'react-dom/client';
import './index.css';
import App from './App';

// biome-ignore lint/style/noNonNullAssertion:
ReactDOM.createRoot(document.getElementById('root')!).render(
	<StrictMode>
		<App />
	</StrictMode>,
);
