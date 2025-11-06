import React from 'react';
import ReactDOM from 'react-dom/client';
import InterventionOverlay from './components/InterventionOverlay.tsx';
// (필요시 이 창 전용 CSS import)
// import './overlay.css'; 

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <InterventionOverlay />
  </React.StrictMode>
);