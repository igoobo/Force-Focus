import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.jsx'
import { GoogleOAuthProvider } from '@react-oauth/google'

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <GoogleOAuthProvider clientId="1050651183992-2j8c6pidjaq53vt6tauovjt1e9blsman.apps.googleusercontent.com">
      <App />
    </GoogleOAuthProvider>
  </StrictMode>
)